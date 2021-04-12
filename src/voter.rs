use std::fmt::Display;

use crate::voting::VotingRounds;
use crate::chain::Chain;

pub struct Voter {
	pub id: String,
	pub chain: Chain,
	pub voting_rounds: VotingRounds,
}

impl Voter {
    pub fn new(id: &str, chain: Chain, voting_rounds: VotingRounds) -> Self {
		Self {
			id: id.to_string(),
			chain,
			voting_rounds,
		}
	}

	pub fn list_commits(&self) {
		for c in self.chain.commits() {
			println!("{}", &c.1);
		}
	}
}

impl Display for Voter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}
