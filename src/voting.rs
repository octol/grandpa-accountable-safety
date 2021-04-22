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
	block::BlockNumber,
	chain::Chain,
	protocol::{Equivocation, EquivocationDetected, QueryResponse},
	voter::{VoterId, VoterName},
};
use itertools::Itertools;
use std::{
	collections::{HashMap, HashSet},
	fmt::{Display, Formatter},
};

#[derive(Clone, Debug)]
pub struct VoterSet {
	// WIP: consider store as VoterId to avoid ugly conversions
	pub voters: HashSet<VoterName>,
}

impl VoterSet {
	pub fn new(voter_ids: &[VoterName]) -> Self {
		Self {
			voters: voter_ids.iter().cloned().collect(),
		}
	}

	pub fn is_member(&self, voter: VoterName) -> bool {
		self.voters.contains(voter)
	}

	pub fn voter_ids(&self) -> Vec<VoterId> {
		self.voters.iter().map(|v| String::from(*v)).collect()
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

impl Default for VotingRounds {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone, Debug)]
pub struct VotingRound {
	pub round_number: RoundNumber,
	pub voter_set: VoterSet,
	pub prevotes: Vec<Prevote>,
	pub precommits: Vec<Precommit>,
	pub finalized: Option<BlockNumber>,
	// We might have multiple voting rounds per round when the network is forked. This field is used
	// to disambiguate them
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
			.iter()
			.map(|(n, id)| {
				assert!(self.voter_set.is_member(id));
				Prevote::new(*n, id)
			})
			.collect::<Vec<_>>();
		self.prevotes.append(&mut votes);
	}

	pub fn precommit(&mut self, votes: &[(BlockNumber, VoterName)]) {
		let mut votes = votes
			.iter()
			.map(|(n, id)| {
				assert!(self.voter_set.is_member(id));
				Precommit::new(*n, id)
			})
			.collect::<Vec<_>>();
		self.precommits.append(&mut votes);
	}
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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

pub trait Vote: std::hash::Hash + Eq {
	fn id(&self) -> VoterName;

	fn target(&self) -> BlockNumber;
}

impl Vote for Prevote {
	fn id(&self) -> VoterName {
		self.id
	}

	fn target(&self) -> BlockNumber {
		self.target_number
	}
}

impl Vote for Precommit {
	fn id(&self) -> VoterName {
		self.id
	}

	fn target(&self) -> BlockNumber {
		self.target_number
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

	pub fn names(&self) -> impl Iterator<Item = VoterName> + '_ {
		self.precommits.iter().map(|precommit| precommit.id)
	}

	pub fn ids(&self) -> impl Iterator<Item = VoterId> + '_ {
		self.precommits
			.iter()
			.map(|precommit| precommit.id.to_string())
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

// Check the validity of a response.
// The purpose of the response is to return a set of votes showing it is impossible to have a
// supermajority for the given block.
pub fn check_query_reply_is_valid(
	response: &QueryResponse,
	block: BlockNumber,
	voters: &[VoterId],
	chain: &Chain,
) -> Option<EquivocationDetected> {
	let unique_voters: HashSet<VoterId> = response
		.ids()
		.into_iter()
		.map(|id| id.to_string())
		.unique()
		.collect();

	let num_equivocations_in_response =
		response.ids().iter().count() - unique_voters.iter().count();
	if num_equivocations_in_response > 0 {
		todo!("Equivocation detected!");
	}

	// Check impossible to have supermajority for the block
	let prevotes_includes_block = response
		.target_numbers()
		.into_iter()
		.filter(|target_number| chain.block_includes(*target_number, block))
		.count();

	// + Add absent votes
	let voters = voters.iter().cloned().collect::<HashSet<_>>();
	let num_voters = voters.len();
	let absent_voters = voters.difference(&unique_voters).count();

	// A valid response has votes showing it's impossible to have supermajority for the earlier
	// finalized block on the other branch
	if 3 * (prevotes_includes_block + absent_voters) <= 2 * num_voters {
		None
	} else {
		// WIP: return a proper response.
		// We can't have a todo! here as the Byzantine voter logic uses the return value to
		// determine which response to send.
		Some(EquivocationDetected::InvalidResponse(
			"placeholder".to_string(),
		))
	}
}

pub fn cross_check_votes<V: Vote>(votes0: Vec<V>, votes1: Vec<V>) -> Option<Vec<Equivocation>> {
	// Take the union
	let votes0: HashSet<_> = votes0.iter().collect();
	let votes1: HashSet<_> = votes1.iter().collect();
	let union: HashSet<_> = votes0.union(&votes1).collect();

	let mut unique_ids: Vec<_> = union.iter().map(|vote| vote.id()).unique().collect();
	unique_ids.sort();

	// Find any duplicate id in the union
	let mut equivocations = Vec::new();
	for id in unique_ids {
		let duplicates: Vec<_> = union.iter().filter(|vote| vote.id() == id).collect();
		if duplicates.len() > 1 {
			let mut duplicate_blocks: Vec<_> =
				duplicates.iter().map(|vote| vote.target()).collect();
			duplicate_blocks.sort();
			println!(
				"Equivocation detected: voter {} for blocks {:?}",
				id, duplicate_blocks,
			);

			let new_equivocation = Equivocation {
				voter: id.to_string(),
				blocks: duplicate_blocks,
			};

			equivocations.push(new_equivocation);
		}
	}

	if equivocations.is_empty() {
		None
	} else {
		// Some(EquivocationDetected::Prevote(equivocations))
		Some(equivocations)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn cross_check_votes_without_equivocations() {
		let precommits = vec![
			Precommit {
				target_number: 1,
				id: "Alice",
			},
			Precommit {
				target_number: 1,
				id: "Bob",
			},
		];
		let commit = Commit {
			target_number: 1,
			precommits: vec![
				Precommit {
					target_number: 1,
					id: "Alice",
				},
				Precommit {
					target_number: 1,
					id: "Bob",
				},
			],
		};
		assert_eq!(cross_check_votes(precommits, commit.precommits), None);
	}

	#[test]
	fn cross_check_votes_with_equivocations() {
		let precommits = vec![
			Precommit {
				target_number: 1,
				id: "Alice",
			},
			Precommit {
				target_number: 1,
				id: "Bob",
			},
		];
		let commit = Commit {
			target_number: 1,
			precommits: vec![
				Precommit {
					target_number: 2,
					id: "Alice",
				},
				Precommit {
					target_number: 1,
					id: "Bob",
				},
			],
		};
		assert_eq!(
			cross_check_votes(precommits, commit.precommits),
			Some(vec![Equivocation {
				voter: "Alice".to_string(),
				blocks: vec![1, 2],
			}]),
		)
	}
}
