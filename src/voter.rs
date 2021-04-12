use std::collections::VecDeque;
use std::fmt::Display;

use crate::voting::VotingRounds;
use crate::chain::Chain;
use crate::action::Action;

pub struct Voter {
	pub id: String,
	pub chain: Chain,
	pub voting_rounds: VotingRounds,
	pub actions: VecDeque<(usize, Action)>,
}

impl Voter {
    pub fn new(id: &str, chain: Chain, voting_rounds: VotingRounds) -> Self {
		Self {
			id: id.to_string(),
			chain,
			voting_rounds,
			actions: Default::default(),
		}
	}

	pub fn list_commits(&self) {
		for c in self.chain.commits() {
			println!("{}", &c.1);
		}
	}

	pub fn process_actions(&self, current_tick: usize) {
		if let Some = self.actions.front() {
		}
		todo!();
	}
}

impl Display for Voter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}
