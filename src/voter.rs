use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::fmt::Display;

use crate::voting::{VoterSet, VotingRounds, Commit};
use crate::chain::Chain;
use crate::action::Action;

#[derive(Debug)]
pub enum Request {
	SendCommit(Commit),
}

pub struct Voter {
	pub id: String,
	pub chain: Chain,
	pub voter_set: VoterSet,
	pub voting_rounds: VotingRounds,
	pub actions: BTreeMap<usize, Action>,
}

impl Voter {
    pub fn new(id: &str, chain: Chain, voter_set: VoterSet, voting_rounds: VotingRounds) -> Self {
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

	pub fn process_actions(&mut self, current_tick: usize) -> Vec<(String, Request)> {
		let mut actions = self.actions.split_off(&current_tick);
		std::mem::swap(&mut actions, &mut self.actions);

		let mut requests = Vec::new();
		for action in actions {
			match action.1 {
				Action::BroadcastCommits => {
					for voter in &self.voter_set.voters {
						if *voter != self.id {
							for c in self.commits() {
								requests.push((voter.to_string(), Request::SendCommit(c.1.clone())));
							}
						}
					}
				},
			}
		}
		requests
	}

	pub fn handle_request(&mut self, request: Request) {
		match request {
			Request::SendCommit(commit) => {
				println!("{}: received: {}", self.id, commit);

				for (_block_number, previous_commit) in self.chain.commits() {
					if !self.chain.is_descendent(commit.target_number, previous_commit.target_number) {
						println!("{}: received Commit is not descendent of last finalized", self.id);
					}
				}
			}
		}
	}
}

impl Display for Voter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::BTreeMap;

	#[test]
	fn split_off_actions() {
		let mut actions = BTreeMap::new();
		actions.insert(0, Action::BroadcastCommit);
		actions.insert(1, Action::BroadcastCommit);
		actions.insert(2, Action::BroadcastCommit);
		actions.insert(3, Action::BroadcastCommit);

		let mut a = actions.split_off(&2);
		std::mem::swap(&mut a, &mut actions);

		for b in a {
			dbg!(&b.0);
		}
	}
}
