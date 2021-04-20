// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{
	action::{Action, TriggerAtTick},
	chain::Chain,
	message::{Message, Payload, Request, Response},
	protocol::{AccountableSafety, EquivocationDetected, Query},
	voting::{check_prevote_reply_is_valid, check_precommit_reply_is_valid, Commit, VoterSet, VotingRounds},
};
use std::{collections::HashMap, fmt::Display};

pub type VoterName = &'static str;
pub type VoterId = String;

pub struct Voter {
	pub id: VoterId,
	pub chain: Chain,
	pub voter_set: VoterSet,
	pub voting_rounds: VotingRounds,
	pub actions: Vec<(TriggerAtTick, Action)>,
	pub accountable_safety: Vec<AccountableSafety>,
	pub behaviour: Option<Behaviour>,
}

// If present, controls the behavior of primaritly misbehaving entities
pub enum Behaviour {
	ReturnPrecommits,
	ReturnPrevotes,
}

impl Voter {
	pub fn new(
		id: VoterId,
		chain: Chain,
		voter_set: VoterSet,
		voting_rounds: VotingRounds,
		behaviour: Option<Behaviour>,
	) -> Self {
		Self {
			id,
			chain,
			voter_set,
			voting_rounds,
			actions: Default::default(),
			accountable_safety: Default::default(),
			behaviour,
		}
	}

	pub fn add_actions(&mut self, actions: Vec<(usize, Action)>) {
		for (tick, action) in actions {
			self.actions.push((tick, action));
		}
	}

	pub fn list_commits(&self) {
		for c in self.chain.commits() {
			println!("{}", &c.1);
		}
	}

	pub fn commits(&self) -> &HashMap<u32, Commit> {
		self.chain.commits()
	}

	pub fn process_actions(&mut self, current_tick: usize) -> Vec<Message> {
		let actions = self
			.actions
			.iter()
			.filter(|a| a.0 <= current_tick)
			.cloned()
			.collect::<Vec<_>>();
		self.actions.retain(|a| a.0 > current_tick);

		let mut messages = Vec::new();
		for (trigger_time, ref action) in actions {
			match action {
				Action::BroadcastCommits => {
					println!("{}: broadcasting all our commits to all voters", self.id);
					for voter in &self.voter_set.voters {
						if *voter != self.id {
							for commit in self.commits().values() {
								let round =
									self.chain.finalized_round(commit.target_number).unwrap();
								messages.push(Message {
									sender: self.id.clone(),
									receiver: voter.to_string(),
									content: Payload::Request(Request::HereIsCommit(
										*round,
										commit.clone(),
									)),
								});
							}
						}
					}
				}
				Action::SendBlock(id, block_number) => {
					println!("{}: send block {} to {}", self.id, block_number, id);
					let blocks = self.chain.get_chain_of_blocks(*block_number);
					if !blocks.is_empty() {
						messages.push(Message {
							sender: self.id.clone(),
							receiver: id.clone(),
							content: Payload::Request(Request::HereAreBlocks(blocks)),
						});
					} else {
						println!(
							"{}: failed to send block {} as it's not in our chain",
							self.id, block_number
						);
					}
				}
				Action::RequeueRequest((sender, request)) => {
					let should_queue_up = match request {
						Request::HereIsCommit(_round, commit) => {
							self.chain.knows_about_block(commit.target_number)
						}
						_ => true,
					};
					if should_queue_up {
						messages.push(Message {
							sender: sender.clone(),
							receiver: self.id.clone(),
							content: Payload::Request(request.clone()),
						});
					} else {
						// Postpone
						self.actions.push((trigger_time + 10, action.clone()));
					}
				}
				Action::AskVotersAboutEstimate(query) => {
					let Query {
						round,
						receivers,
						block_not_included,
					} = query;

					let _round_for_block_not_included =
						self.chain.finalized_round(*block_not_included).unwrap();

					for receiver in receivers {
						println!(
							"{}: asking {} about block {} and round {}",
							self.id, receiver, block_not_included, round,
						);
						let msg = Message {
							sender: self.id.clone(),
							receiver: receiver.clone(),
							content: Payload::Request(
								Request::WhyDidEstimateForRoundNotIncludeBlock(
									*round,
									*block_not_included,
								),
							),
						};
						messages.push(msg);
					}
				}
			}
		}
		messages
	}

	pub fn handle_request(
		&mut self,
		request: (VoterId, Request),
		current_tick: usize,
	) -> Vec<(VoterId, Response)> {
		match request.1 {
			Request::HereIsCommit(round_number, ref commit) => {
				// Ignore commits we already know about
				if let Some(chain_commit) = self.chain.commit_for_block(commit.target_number) {
					assert_eq!(commit, chain_commit);
					return Default::default();
				}
				println!("{}: received {}", self.id, commit);

				if !self.chain.knows_about_block(commit.target_number) {
					// Requeue request for later, when we hopefully know about the block
					self.actions
						.push((current_tick + 10, Action::RequeueRequest(request.clone())));
					println!("{}: requesting block {}", self.id, commit.target_number);
					return vec![(request.0, Response::RequestBlock(commit.target_number))];
				}

				for previous_commit in self.chain.commits().values() {
					if !self
						.chain
						.is_descendent(commit.target_number, previous_commit.target_number)
					{
						println!(
							"{}: received commit is not descendent of last finalized, \
							triggering accountable safety protocol!",
							self.id
						);

						let block_not_included = previous_commit.target_number;
						let round_for_block_not_included =
							self.chain.finalized_round(block_not_included).unwrap();
						let commit_for_block_not_included = previous_commit;

						let mut accountable_safety_instance = AccountableSafety::start(
							block_not_included,
							*round_for_block_not_included,
							commit_for_block_not_included.clone(),
						);

						let voters_in_precommit = commit
							.precommits
							.iter()
							.map(|pc| pc.id.to_string())
							.collect::<Vec<VoterId>>();

						let round_for_new_block = round_number;
						let query = accountable_safety_instance
							.start_query_round(round_for_new_block, voters_in_precommit);

						self.accountable_safety.push(accountable_safety_instance);
						self.actions
							.push((current_tick + 10, Action::AskVotersAboutEstimate(query)));
					}
				}
			}
			Request::HereAreBlocks(blocks) => {
				println!("{}: received blocks", self.id);
				for block in blocks {
					if let Some(chain_block) = self.chain.get_block(block.number) {
						assert_eq!(&block, chain_block);
					} else {
						println!("{}: adding block {}", self.id, block);
						self.chain.add_block(block);
					}
				}
			}
			Request::WhyDidEstimateForRoundNotIncludeBlock(round, block_not_included) => {
				// This is a container of voting rounds, since some voters might have equivocated
				// and have multiple parallel sets of histories that it presents to different
				// voters.
				let voting_rounds_for_previous_block =
					self.voting_rounds.get(&(round - 1)).unwrap();

				match self.behaviour {
					Some(Behaviour::ReturnPrecommits) | None => {
						// Now if this is a equivocating voter, they will want to return the set of
						// commits corresponding to the valid round.
						//
						// A simple way to make this choice is by checking which of the sets of
						// precommits are considered valid
						let valid_voting_round: Vec<_> = voting_rounds_for_previous_block
							.iter()
							.filter(|voting_round| {
								check_precommit_reply_is_valid(
									&voting_round.precommits,
									block_not_included,
									&self.voter_set.voter_ids(),
									&self.chain,
								)
								.is_none()
							})
							.collect();
						assert_eq!(valid_voting_round.len(), 1);
						let valid_voting_round = valid_voting_round.into_iter().next().unwrap().clone();

						return vec![(
							request.0,
							Response::PrecommitsForEstimate(round, valid_voting_round.precommits),
						)];
					}
					Some(Behaviour::ReturnPrevotes) => {
						let valid_voting_round: Vec<_> = voting_rounds_for_previous_block
							.iter()
							.filter(|voting_round| {
								check_prevote_reply_is_valid(
									&voting_round.prevotes,
									block_not_included,
									&self.voter_set.voter_ids(),
									&self.chain,
								)
								.is_none()
							})
							.collect();
						assert_eq!(valid_voting_round.len(), 1);
						let valid_voting_round = valid_voting_round.into_iter().next().unwrap().clone();

						return vec![(
							request.0,
							Response::PrevotesForEstimate(round, valid_voting_round.prevotes),
						)];
					}
				}
			}
		}
		Default::default()
	}

	pub fn handle_response(&mut self, response: (VoterId, Response), current_tick: usize) {
		match response.1 {
			Response::RequestBlock(block_number) => {
				self.actions.push((
					current_tick + 10,
					Action::SendBlock(response.0, block_number),
				));
			}
			Response::PrecommitsForEstimate(round_number, ref precommits) => {
				println!(
					"{}: handle PrecommitsForEstimate from {}: {}, {:?}",
					self.id, response.0, round_number, precommits
				);
				// WIP: assume a single instance
				let next_query = self
					.accountable_safety
					.iter_mut()
					.next()
					.unwrap()
					.add_response(round_number, response.0, precommits.clone(), &self.chain);

				if let Some(next_query) = next_query {
					self.actions.push((
						current_tick + 10,
						Action::AskVotersAboutEstimate(next_query),
					));
				}
			}
			Response::PrevotesForEstimate(round_number, ref prevotes) => {
				todo!();
			}
		}
	}

	pub fn equivocations_detected(&self) -> Vec<EquivocationDetected> {
		self.accountable_safety
			.iter()
			.flat_map(|acc_safety| acc_safety.equivocations_detected())
			.collect()
	}
}

impl Display for Voter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}
