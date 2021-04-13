use crate::VoterId;
use crate::block::{Block, BlockNumber};
use crate::VoterName;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Display;

use crate::action::Action;
use crate::chain::Chain;
use crate::voting::{Commit, VoterSet, VotingRounds};

#[derive(Debug)]
pub enum Request {
	SendCommit(Commit),
	SendBlock(Block),
}

#[derive(Debug)]
pub enum Response {
	RequestBlock(BlockNumber),
}

pub struct Voter {
	pub id: VoterId,
	pub chain: Chain,
	pub voter_set: VoterSet,
	pub voting_rounds: VotingRounds,
	pub actions: BTreeMap<usize, Action>,
}

impl Voter {
	pub fn new(
		id: VoterName,
		chain: Chain,
		voter_set: VoterSet,
		voting_rounds: VotingRounds,
	) -> Self {
		Self {
			id: id.to_string(),
			chain,
			voter_set,
			voting_rounds,
			actions: Default::default(),
		}
	}

	pub fn add_actions(&mut self, actions: Vec<(usize, Action)>) {
		for (tick, action) in actions {
			self.actions.insert(tick, action);
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

	pub fn process_actions(&mut self, current_tick: usize) -> Vec<(VoterId, Request)> {
		let mut actions = self.actions.split_off(&current_tick);
		std::mem::swap(&mut actions, &mut self.actions);

		let mut requests = Vec::new();
		for action in actions {
			match action.1 {
				Action::BroadcastCommits => {
					println!("{}: Broadcasting all our commits to all voters", self.id);
					for voter in &self.voter_set.voters {
						if *voter != self.id {
							for c in self.commits() {
								requests
									.push((voter.to_string(), Request::SendCommit(c.1.clone())));
							}
						}
					}
				}
				Action::SendBlock(id, block_number) => {
					println!("{}: Send block {} to {}", self.id, block_number, id);
					if let Some(block) = self.chain.get_block(block_number) {
						requests.push((id, Request::SendBlock(block.clone())));
					} else {
						println!(
							"{}: failed to send block {} as it's not in our chain",
							self.id,
							block_number
						);
					}
				},
			}
		}
		requests
	}

	pub fn handle_request(&mut self, request: (VoterId, Request)) -> Vec<(VoterId, Response)> {
		match request.1 {
			Request::SendCommit(commit) => {
				if let Some(chain_commit) = self.chain.commit_for_block(commit.target_number) {
					assert_eq!(&commit, chain_commit);
					return Default::default();
				}
				println!("{}: received {}", self.id, commit);

				if !self.chain.knows_about_block(commit.target_number) {
					// TODO: re-queue request with a delay
					println!("{}: requesting block {}", self.id, commit.target_number);
					return vec![(request.0, Response::RequestBlock(commit.target_number))];
				}

				for (_block_number, previous_commit) in self.chain.commits() {
					if !self
						.chain
						.is_descendent(commit.target_number, previous_commit.target_number)
					{
						println!(
							"{}: received Commit is not descendent of last finalized",
							self.id
						);
					}
				}
			}
			Request::SendBlock(block) => {
				if let Some(chain_block) = self.chain.get_block(block.number) {
					assert_eq!(&block, chain_block);
					return Default::default();
				}
				println!("{}: received {}", self.id, block);
			},
		}
		Default::default()
	}

	pub fn handle_response(&mut self, response: (VoterId, Response), current_tick: usize) {
		match response.1 {
			Response::RequestBlock(block_number) => {
				self.actions.insert(current_tick + 1, Action::SendBlock(response.0, block_number));
			}
		}
	}
}

impl Display for Voter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}
