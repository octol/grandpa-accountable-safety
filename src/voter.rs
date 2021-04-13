use crate::block::{Block, BlockNumber};
use crate::VoterId;
use crate::VoterName;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Display;

use crate::action::Action;
use crate::chain::Chain;
use crate::voting::{Commit, VoterSet, VotingRounds};

#[derive(Debug, Clone)]
pub enum Request {
	SendCommit(Commit),
	SendBlock(Block),
}

#[derive(Debug, Clone)]
pub enum Response {
	RequestBlock(BlockNumber),
}

#[derive(Debug)]
pub enum Payload {
	Request(Request),
	Response(Response),
}

#[derive(Debug)]
pub struct Message {
	pub sender: VoterId,
	pub receiver: VoterId,
	pub content: Payload,
}

pub struct Voter {
	pub id: VoterId,
	pub chain: Chain,
	pub voter_set: VoterSet,
	pub voting_rounds: VotingRounds,
	pub actions: Vec<(usize, Action)>,
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
		let actions = self.actions.iter().filter(|a| a.0 <= current_tick).cloned().collect::<Vec<_>>();
		self.actions.retain(|a| a.0 > current_tick);

		let mut messages = Vec::new();
		for (_trigger_time, action) in actions {
			match action {
				Action::BroadcastCommits => {
					println!("{}: broadcasting all our commits to all voters", self.id);
					for voter in &self.voter_set.voters {
						if *voter != self.id {
							for (_, c) in self.commits() {
								messages.push(Message {
									sender: self.id.clone(),
									receiver: voter.to_string(),
									content: Payload::Request(Request::SendCommit(c.clone())),
								});
							}
						}
					}
				}
				Action::SendBlock(id, block_number) => {
					println!("{}: send block {} to {}", self.id, block_number, id);
					if let Some(block) = self.chain.get_block(block_number) {
						messages.push(Message {
							sender: self.id.clone(),
							receiver: id,
							content: Payload::Request(Request::SendBlock(block.clone())),
						});
					} else {
						println!(
							"{}: failed to send block {} as it's not in our chain",
							self.id, block_number
						);
					}
				}
				Action::RequeueRequest((sender, request)) => {
					//println!("{}: requeue ({}, {:?})", self.id, sender, request);
					messages.push(Message {
						sender,
						receiver: self.id.clone(),
						content: Payload::Request(request),
					});
				}
				_ => todo!(),
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
			Request::SendCommit(ref commit) => {
				// Ignore commits we already know about
				if let Some(chain_commit) = self.chain.commit_for_block(commit.target_number) {
					assert_eq!(commit, chain_commit);
					return Default::default();
				}
				println!("{}: received {}", self.id, commit);

				if !self.chain.knows_about_block(commit.target_number) {
					// Requeue request for later, when we hopefully know about the block
					//println!("{}: saving as requeue request action: ({}, {:?})", self.id, current_tick + 10, request);
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
							should trigger accountable safety protocol!",
							self.id
						);
					}
				}
			}
			Request::SendBlock(ref block) => {
				// Ignore blocks we alreday know about
				if let Some(chain_block) = self.chain.get_block(block.number) {
					assert_eq!(block, chain_block);
					return Default::default();
				}
				println!("{}: received {}", self.id, block);

				if !self.chain.knows_about_block(block.parent) {
					// Requeue request for later, when we hopefully know about block
					//println!("{}: saving as requeue request action: ({}, {:?})", self.id, current_tick + 10, request);
					self.actions
						.push((current_tick + 10, Action::RequeueRequest(request.clone())));
					println!("{}: requesting block {}", self.id, block.parent);
					return vec![(request.0, Response::RequestBlock(block.parent))];
				}

				println!("{}: adding block {}", self.id, block);
				self.chain.add_block(block.clone());
			}
		}
		Default::default()
	}

	pub fn handle_response(&mut self, response: (VoterId, Response), current_tick: usize) {
		match response.1 {
			Response::RequestBlock(block_number) => {
				self.actions.push((
					current_tick + 1,
					Action::SendBlock(response.0, block_number),
				));
			}
		}
	}
}

impl Display for Voter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}
