use std::collections::HashSet;

use crate::block::BlockNumber;

type VoterId = &'static str;

#[derive(Clone)]
pub struct VoterSet {
	pub voters: HashSet<VoterId>,
}

impl VoterSet {
	pub fn new(voter_ids: &[VoterId]) -> Self {
		Self {
			voters: voter_ids.into_iter().cloned().collect(),
		}
	}

	pub fn is_member(&self, voter: VoterId) -> bool {
		self.voters.contains(voter)
	}
}

pub struct VotingRound {
	pub round_number: u64,
	pub voter_set: VoterSet,
	pub prevotes: Vec<Prevote>,
	pub precommits: Vec<Precommit>,
}

impl VotingRound {
	pub fn new(round_number: u64, voter_set: VoterSet) -> Self {
		Self {
			round_number,
			voter_set,
			prevotes: Default::default(),
			precommits: Default::default(),
		}
	}

	pub fn prevote(&mut self, votes: &[(BlockNumber, VoterId)]) {
		let mut votes = votes
			.into_iter()
			.map(|(n, id)| {
				assert!(self.voter_set.is_member(id));
				Prevote::new(*n, id)
			})
			.collect::<Vec<_>>();
		self.prevotes.append(&mut votes);
	}

	pub fn precommit(&mut self, votes: &[(BlockNumber, VoterId)]) {
		let mut votes = votes
			.into_iter()
			.map(|(n, id)| {
				assert!(self.voter_set.is_member(id));
				Precommit::new(*n, id)
			})
			.collect::<Vec<_>>();
		self.precommits.append(&mut votes);
	}
}

#[derive(Clone)]
pub struct Prevote {
	pub target_number: BlockNumber,
	pub id: VoterId,
}

impl Prevote {
	pub fn new(target_number: BlockNumber, id: VoterId) -> Self {
		Self { target_number, id }
	}
}

#[derive(Clone)]
pub struct Precommit {
	pub target_number: BlockNumber,
	pub id: VoterId,
}

impl Precommit {
	pub fn new(target_number: BlockNumber, id: VoterId) -> Self {
		Self { target_number, id }
	}
}

pub struct Commit {
	target_number: BlockNumber,
	precommits: Vec<Precommit>,
}

impl Commit {
	pub fn new(target_number: BlockNumber, precommits: Vec<Precommit>) -> Self {
		Self {
			target_number,
			precommits,
		}
	}
}
