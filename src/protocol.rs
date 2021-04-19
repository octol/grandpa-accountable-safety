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
	voter::VoterId,
	voting::{
		cross_check_precommit_reply_against_commit, precommit_reply_is_valid2, Commit, Precommit,
		RoundNumber,
	},
};
use itertools::Itertools;
use std::collections::BTreeMap;

// State of the accountable safety protocol
#[derive(Debug)]
pub struct AccountableSafety {
	block_not_included: BlockNumber,
	round_for_block_not_included: RoundNumber,
	commit_for_block_not_included: Commit,
	querying_rounds: BTreeMap<RoundNumber, QueryState>,
}

// The state of the querying about a specific round.
// The query is about why in the given round didn't the estimate for the previous round not include
// `block_not_included`.
#[derive(Debug)]
struct QueryState {
	round: RoundNumber,
	voters: Vec<VoterId>,
	responses: BTreeMap<VoterId, Vec<Precommit>>,
}

impl QueryState {
	fn add_response(&mut self, voter: VoterId, precommits: Vec<Precommit>) {
		self.responses.insert(voter, precommits);
	}
}

// Query sent to the voters for a specific round
#[derive(Debug, Clone)]
pub struct Query {
	pub round: RoundNumber,
	pub receivers: Vec<VoterId>,
	pub block_not_included: BlockNumber,
}

impl AccountableSafety {
	pub fn start(
		block_not_included: BlockNumber,
		round_for_block_not_included: RoundNumber,
		commit_for_block_not_included: Commit,
	) -> Self {
		Self {
			block_not_included,
			round_for_block_not_included,
			commit_for_block_not_included,
			querying_rounds: Default::default(),
		}
	}

	// Ask the question why the estimate for the previous round didn't include the earlier block
	pub fn start_query_round(&mut self, round: RoundNumber, voters: Vec<VoterId>) -> Query {
		// QueryState will keep track of responses that return
		self.querying_rounds.insert(
			round,
			QueryState {
				round,
				voters: voters.clone(),
				responses: Default::default(),
			},
		);

		Query {
			round,
			receivers: voters,
			block_not_included: self.block_not_included,
		}
	}

	pub fn add_response(
		&mut self,
		round: RoundNumber,
		voter: VoterId,
		precommits: Vec<Precommit>,
		chain: &Chain,
	) -> Option<Query> {
		// Add response to the right QueryState in querying_rounds.
		{
			let mut querying_state = self.querying_rounds.get_mut(&round).unwrap();
			let voters = querying_state.voters.clone();
			precommit_reply_is_valid2(&precommits, self.block_not_included, &voters, &chain);
			querying_state.add_response(voter, precommits.clone());
		}

		// Was this for the round directly after the round where the block that should have been
		// included, but wasn't, was finalized?
		if round == self.round_for_block_not_included + 1 {
			cross_check_precommit_reply_against_commit(
				&precommits,
				self.commit_for_block_not_included.clone(),
			);
		} else {
			// Start the next round if not already done
			let next_round_to_investigate = round - 1;

			// WIP(JON): more receivers might show up in later responses.
			if !self
				.querying_rounds
				.contains_key(&next_round_to_investigate)
			{
				let voters_in_precommits: Vec<_> = precommits
					.iter()
					.map(|pre| pre.id.to_string())
					.unique()
					.collect();
				return Some(
					self.start_query_round(next_round_to_investigate, voters_in_precommits),
				);
			}
		}

		return None;
	}
}
