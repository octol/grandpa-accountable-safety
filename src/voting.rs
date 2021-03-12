use std::collections::HashSet;

use crate::block::BlockNumber;

type VoterId = u8;

pub struct VoterSet {
    pub voters: HashSet<VoterId>,
}

impl VoterSet {
    pub fn new(voter_ids: &[VoterId]) -> Self {
        Self {
            voters: voter_ids.iter().cloned().collect(),
        }
    }
}

pub struct VotingRound {
    pub round_number: u64,
    pub prevotes: Vec<Prevote>,
    pub precommits: Vec<Precommit>,
}

impl VotingRound {
    pub fn new(round_number: u64) -> Self {
        Self {
            round_number,
            prevotes: Default::default(),
            precommits: Default::default(),
        }
    }
}

pub struct Prevote {
    pub target_number: BlockNumber,
    pub id: VoterId,
}

impl Prevote {
    pub fn new(target_number: BlockNumber, id: VoterId) -> Self {
        Self { target_number, id }
    }
}

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
