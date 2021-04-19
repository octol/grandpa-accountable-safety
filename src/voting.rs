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
	chain::Chain,
	block::BlockNumber,
	voter::{VoterId, VoterName},
};
use itertools::Itertools;
use std::{
	collections::{HashMap, HashSet},
	fmt::{Display, Formatter},
};

#[derive(Clone, Debug)]
pub struct VoterSet {
	pub voters: HashSet<VoterName>,
}

impl VoterSet {
	pub fn new(voter_ids: &[VoterName]) -> Self {
		Self {
			voters: voter_ids.into_iter().cloned().collect(),
		}
	}

	pub fn is_member(&self, voter: VoterName) -> bool {
		self.voters.contains(voter)
	}
}

pub type RoundNumber = u64;

#[derive(Clone, Debug)]
pub struct VotingRounds(pub HashMap<RoundNumber, Vec<VotingRound>>);

impl VotingRounds {
	pub fn new() -> Self {
		Self(HashMap::new())
	}

	pub fn get(&self, round_number: &RoundNumber) -> Option<&Vec<VotingRound>> {
		self.0.get(round_number)
	}

	pub fn add(&mut self, voting_round: VotingRound) {
		let round_number = voting_round.round_number;
		if let Some(vr) = self.0.get_mut(&round_number) {
			vr.push(voting_round)
		} else {
			self.0.insert(round_number, vec![voting_round]);
		}
	}

	pub fn extend(&mut self, other: VotingRounds) {
		self.0.extend(other.0);
	}
}

#[derive(Clone, Debug)]
pub struct VotingRound {
	pub round_number: RoundNumber,
	pub voter_set: VoterSet,
	pub prevotes: Vec<Prevote>,
	pub precommits: Vec<Precommit>,
	pub finalized: Option<BlockNumber>,
	// We might have multiple voting rounds per round when the network is forked. This field is used to disambiguate
	// them
	pub tag: u32,
}

impl VotingRound {
	pub fn new(round_number: RoundNumber, voter_set: VoterSet) -> Self {
		Self {
			round_number,
			voter_set,
			prevotes: Default::default(),
			precommits: Default::default(),
			finalized: None,
			tag: 0,
		}
	}

	pub fn new_with_tag(round_number: RoundNumber, voter_set: VoterSet, tag: u32) -> Self {
		Self {
			round_number,
			voter_set,
			prevotes: Default::default(),
			precommits: Default::default(),
			finalized: None,
			tag,
		}
	}

	pub fn prevote(&mut self, votes: &[(BlockNumber, VoterName)]) {
		let mut votes = votes
			.into_iter()
			.map(|(n, id)| {
				assert!(self.voter_set.is_member(id));
				Prevote::new(*n, id)
			})
			.collect::<Vec<_>>();
		self.prevotes.append(&mut votes);
	}

	pub fn precommit(&mut self, votes: &[(BlockNumber, VoterName)]) {
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

#[derive(Clone, Debug)]
pub struct Prevote {
	pub target_number: BlockNumber,
	pub id: VoterName,
}

impl Prevote {
	pub fn new(target_number: BlockNumber, id: VoterName) -> Self {
		Self { target_number, id }
	}
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Precommit {
	pub target_number: BlockNumber,
	pub id: VoterName,
}

impl Precommit {
	pub fn new(target_number: BlockNumber, id: VoterName) -> Self {
		Self { target_number, id }
	}
}

impl Display for Precommit {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(
			f,
			"Precommit {{ target_number: {}, id: {} }}",
			self.target_number, self.id
		)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
	pub target_number: BlockNumber,
	pub precommits: Vec<Precommit>,
}

impl Commit {
	pub fn new(target_number: BlockNumber, precommits: Vec<Precommit>) -> Self {
		Self {
			target_number,
			precommits,
		}
	}
}

impl Display for Commit {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(
			f,
			"Commit({}, {{ {} }})",
			self.target_number,
			self.precommits.iter().map(|pc| pc.id).format(", ")
		)
	}
}

// Check the validity of a response containing precommits.
// The purpose of the response is to return a set of precommits showing it is impossible to have a
// supermajority for the given block.
pub fn precommit_reply_is_valid(
	response: &Vec<Precommit>,
	block: BlockNumber,
	voter_set: &VoterSet,
	chain: &Chain,
) -> bool {
	// No equivocations
	let unique_voters: HashSet<VoterName> = response.iter().map(|pre| pre.id).unique().collect();
	let num_equivocations_in_commit = response.iter().count() - unique_voters.iter().count();
	assert!(num_equivocations_in_commit == 0);

	// Check impossible to have supermajority for the block
	let precommits_includes_block = response
		.iter()
		.filter(|precommit| chain.block_includes(precommit.target_number, block))
		.count();

	// + Add absent votes
	let num_voters = voter_set.voters.len();
	let absent_voters = voter_set.voters.difference(&unique_voters).count();

	// A valid response has precommits showing it's impossible to have supermajority for the earlier
	// finalized block on the other branch
	!(3 * (precommits_includes_block + absent_voters) > 2 * num_voters)
}

pub fn precommit_reply_is_valid2(
	response: &Vec<Precommit>,
	block: BlockNumber,
	voters: &Vec<VoterId>,
	chain: &Chain,
) -> bool {
	// No equivocations
	let unique_voters: HashSet<VoterId> = response.iter().map(|pre| pre.id.to_string()).unique().collect();
	let num_equivocations_in_commit = response.iter().count() - unique_voters.iter().count();
	assert!(num_equivocations_in_commit == 0);

	// Check impossible to have supermajority for the block
	let precommits_includes_block = response
		.iter()
		.filter(|precommit| chain.block_includes(precommit.target_number, block))
		.count();

	// + Add absent votes
	let voters = voters.iter().cloned().collect::<HashSet<_>>();
	let num_voters = voters.len();
	let absent_voters = voters.difference(&unique_voters).count();

	// A valid response has precommits showing it's impossible to have supermajority for the earlier
	// finalized block on the other branch
	!(3 * (precommits_includes_block + absent_voters) > 2 * num_voters)
}

// Cross check against precommitters in commit message
pub fn cross_check_precommit_reply_against_commit(s: &Vec<Precommit>, commit: Commit) {
	for precommit in &commit.precommits {
		let equivocated_votes: Vec<_> = s.iter().filter(|pre| pre.id == precommit.id).collect();

		if !equivocated_votes.is_empty() {
			print!(
				"Precommit equivocation detected: voter {} for blocks {}",
				precommit.id, precommit.target_number
			);
			equivocated_votes.iter().for_each(|e| {
				print!(", {}", e.target_number);
			});
			print!("\n");
		}
	}
}

pub fn cross_check_prevote_reply_against_prevotes_seen(s: Vec<Prevote>, t: Vec<Prevote>) {
	for prevote in &t {
		let equivocated_votes: Vec<_> = s.iter().filter(|pre| pre.id == prevote.id).collect();

		if !equivocated_votes.is_empty() {
			print!(
				"Prevote equivocation detected: voter {} for blocks {}",
				prevote.id, prevote.target_number
			);
			equivocated_votes.iter().for_each(|e| {
				print!(", {}", e.target_number);
			});
			print!("\n");
		}
	}
}
