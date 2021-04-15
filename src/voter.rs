use crate::{
	action::{Action, TriggerAtTick},
	chain::Chain,
	message::{Message, Payload, Request, Response},
	protocol::{AccountableSafety, Query},
	voting::{precommit_reply_is_valid, Commit, VoterSet, VotingRounds},
	VoterId,
};
use std::{collections::HashMap, fmt::Display};

pub struct Voter {
	pub id: VoterId,
	pub chain: Chain,
	pub voter_set: VoterSet,
	pub voting_rounds: VotingRounds,
	pub actions: Vec<(TriggerAtTick, Action)>,
	pub accountable_safety: Vec<AccountableSafety>,
}

impl Voter {
	pub fn new(
		id: VoterId,
		chain: Chain,
		voter_set: VoterSet,
		voting_rounds: VotingRounds,
	) -> Self {
		Self {
			id,
			chain,
			voter_set,
			voting_rounds,
			actions: Default::default(),
			accountable_safety: Default::default(),
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
							for (_, commit) in self.commits() {
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

					let round_for_block_not_included =
						self.chain.finalized_round(*block_not_included).unwrap();

					for receiver in receivers {
						println!(
							"{}: asking {} about block {}",
							self.id, receiver, block_not_included
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

				for (_block_number, previous_commit) in self.chain.commits() {
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
				// WIP: We can either return Precommits of Prevotes

				// This is a container of voting rounds, since some voters might have equivocated.
				let voting_rounds_for_previous_block =
					self.voting_rounds.get(&(round - 1)).unwrap();

				// Now if this is a equivocating voter, they will want to return the set of commits
				// corresponding to the valid round.
				//
				// We make this choice by checking which of the sets of precommits are considered
				// valid
				let valid_voting_round: Vec<_> = voting_rounds_for_previous_block
					.into_iter()
					.filter(|voting_round| {
						precommit_reply_is_valid(
							&voting_round.precommits,
							block_not_included,
							&self.voter_set,
							&self.chain,
						)
					})
					.collect();
				assert_eq!(valid_voting_round.len(), 1);
				let valid_voting_round = valid_voting_round.into_iter().next().unwrap().clone();

				return vec![(
					request.0,
					Response::PrecommitsForEstimate(
						valid_voting_round.round_number,
						valid_voting_round.precommits
					),
				)];
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
				dbg!(&round_number);
				dbg!(&response);
				// WIP: assume a single instance
				self.accountable_safety
					.iter_mut()
					.next()
					.unwrap()
					.add_response(round_number, response.0, precommits.clone(), &self.chain);
			}
		}
	}
}

impl Display for Voter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}
