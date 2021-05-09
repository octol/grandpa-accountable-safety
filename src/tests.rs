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

// See the documentation for a more detailed description of the scenario and how the protocol plays
// ouit.

use crate::{
	action::Action,
	chain::Chain,
	protocol::{Equivocation, EquivocationDetected},
	voter::{Behaviour, Voter, VoterId},
	voting::{Commit, VoterSet, VotingRound, VotingRounds},
	world::World,
};
use std::collections::BTreeMap;

fn setup_voters_with_two_finalized_forks(behaviour: Behaviour) -> BTreeMap<VoterId, Voter> {
	let names = &["Alice", "Bob", "Carol", "Dave"];
	let voter_set = VoterSet::new(names);

	let mut voters = BTreeMap::new();

	let chain_common = [(1, 0)];
	let chain_a_fork = [(2, 1), (3, 2), (4, 3)];
	let chain_b_fork = [(5, 1), (6, 5), (7, 6), (8, 7)];
	let chain_all: Vec<_> = chain_common
		.iter()
		.chain(chain_a_fork.iter())
		.chain(chain_b_fork.iter())
		.cloned()
		.collect();
	let chain_a: Vec<_> = chain_common
		.iter()
		.chain(chain_a_fork.iter())
		.cloned()
		.collect();
	let chain_b: Vec<_> = chain_common
		.iter()
		.chain(chain_b_fork.iter())
		.cloned()
		.collect();

	// Setup the 4 voters and the voting history that they know about.
	{
		let mut chain = Chain::new_from(&chain_all);
		let mut voting_rounds = create_common_voting_rounds(&voter_set, &mut chain);
		append_voting_rounds_a(&mut voting_rounds, &voter_set, &mut chain);
		append_voting_rounds_b(&mut voting_rounds, &voter_set, &mut chain);
		let id = names[0].to_string();
		voters.insert(
			id.clone(),
			Voter::new(
				id,
				chain.clone(),
				voter_set.clone(),
				voting_rounds,
				Some(behaviour),
			),
		);
	}
	{
		let mut chain = Chain::new_from(&chain_all);
		let mut voting_rounds = create_common_voting_rounds(&voter_set, &mut chain);
		append_voting_rounds_a(&mut voting_rounds, &voter_set, &mut chain);
		append_voting_rounds_b(&mut voting_rounds, &voter_set, &mut chain);
		let id = names[1].to_string();
		voters.insert(
			id.clone(),
			Voter::new(id, chain, voter_set.clone(), voting_rounds, Some(behaviour)),
		);
	}
	{
		let mut chain = Chain::new_from(&chain_a);
		let mut voting_rounds = create_common_voting_rounds(&voter_set, &mut chain);
		append_voting_rounds_a(&mut voting_rounds, &voter_set, &mut chain);
		let id = names[2].to_string();
		voters.insert(
			id.clone(),
			Voter::new(
				id.clone(),
				chain,
				voter_set.clone(),
				voting_rounds,
				Some(behaviour),
			),
		);
	}
	{
		let mut chain = Chain::new_from(&chain_b);
		let mut voting_rounds = create_common_voting_rounds(&voter_set, &mut chain);
		append_voting_rounds_b(&mut voting_rounds, &voter_set, &mut chain);
		let id = names[3].to_string();
		voters.insert(
			id.clone(),
			Voter::new(id, chain, voter_set, voting_rounds, Some(behaviour)),
		);
	}

	// Kick off the simulation by having one voter broadcast all their commits, reveiling the conflicting
	// finalized blocks to the other (honest) voters.
	voters
		.get_mut(&"Dave".to_string())
		.map(|v| v.add_actions(vec![(10, Action::BroadcastCommits)]));

	voters
}

fn create_common_voting_rounds(voter_set: &VoterSet, chain: &mut Chain) -> VotingRounds {
	let mut voting_rounds = VotingRounds::new();
	let voting_round_tag = 0;

	{
		let mut round = VotingRound::new_with_tag(1, voter_set.clone(), voting_round_tag);
		round.prevote(&[(1, "Alice"), (1, "Bob"), (1, "Carol"), (1, "Dave")]);
		round.precommit(&[(1, "Alice"), (1, "Bob"), (1, "Carol"), (1, "Dave")]);
		let commit = Commit::new(1, round.precommits.clone());
		chain.finalize_block(1, round.round_number, commit);
		voting_rounds.add(round);
	}

	voting_rounds
}

// Sequence of voting rounds leading to finalizing block 2 on the first fork
fn append_voting_rounds_a(
	voting_rounds: &mut VotingRounds,
	voter_set: &VoterSet,
	chain: &mut Chain,
) {
	let voting_round_tag = 0;
	{
		let mut round = VotingRound::new_with_tag(2, voter_set.clone(), voting_round_tag);
		round.prevote(&[(4, "Alice"), (4, "Bob"), (2, "Carol")]);
		round.precommit(&[(2, "Alice"), (2, "Bob"), (2, "Carol")]);
		let commit = Commit::new(2, round.precommits.clone());
		chain.finalize_block(2, round.round_number, commit);
		voting_rounds.add(round);
	}
	{
		let mut round = VotingRound::new_with_tag(3, voter_set.clone(), voting_round_tag);
		round.prevote(&[(4, "Alice"), (4, "Bob"), (2, "Carol")]);
		round.precommit(&[(2, "Alice"), (2, "Bob"), (2, "Carol")]);
		voting_rounds.add(round);
	}
	{
		let mut round = VotingRound::new_with_tag(4, voter_set.clone(), voting_round_tag);
		round.prevote(&[(4, "Alice"), (4, "Bob"), (2, "Carol")]);
		round.precommit(&[(2, "Alice"), (2, "Bob"), (2, "Carol")]);
		voting_rounds.add(round);
	}
}

// Sequence of voting rounds leading to finalizing block 8 on the second fork
fn append_voting_rounds_b(
	voting_rounds: &mut VotingRounds,
	voter_set: &VoterSet,
	chain: &mut Chain,
) {
	let voting_round_tag = 1;
	{
		let mut round = VotingRound::new_with_tag(2, voter_set.clone(), voting_round_tag);
		round.prevote(&[(1, "Alice"), (1, "Bob"), (5, "Dave")]);
		round.precommit(&[(1, "Alice"), (1, "Bob"), (1, "Dave")]);
		voting_rounds.add(round);
	}
	{
		let mut round = VotingRound::new_with_tag(3, voter_set.clone(), voting_round_tag);
		round.prevote(&[(1, "Alice"), (1, "Bob"), (5, "Dave")]);
		round.precommit(&[(1, "Alice"), (1, "Bob"), (1, "Dave")]);
		voting_rounds.add(round);
	}
	{
		let mut round = VotingRound::new_with_tag(4, voter_set.clone(), voting_round_tag);
		round.prevote(&[(8, "Alice"), (8, "Bob"), (8, "Dave")]);
		round.precommit(&[(8, "Alice"), (8, "Bob"), (8, "Dave")]);
		let commit = Commit::new(8, round.precommits.clone());
		chain.finalize_block(8, round.round_number, commit);
		voting_rounds.add(round);
	}
}

#[test]
fn basic_example_with_precommits() {
	let mut world = World::new(setup_voters_with_two_finalized_forks(
		Behaviour::ReturnPrecommits,
	));

	world.list_commits();

	println!("\n*** Starting loop ***\n");

	while !world.completed() {
		let requests = world.process_actions();
		let responses = world.handle_requests(requests);
		world.handle_responses(responses);
		world.tick();
	}

	// We get three sets of equivocations, one coming from each voter
	assert_eq!(
		world.equivocations_detected(),
		&[
			EquivocationDetected::Precommit(vec![
				Equivocation {
					voter: "Alice".to_string(),
					blocks: vec![1, 2],
				},
				Equivocation {
					voter: "Bob".to_string(),
					blocks: vec![1, 2],
				}
			]),
			EquivocationDetected::Precommit(vec![
				Equivocation {
					voter: "Alice".to_string(),
					blocks: vec![1, 2],
				},
				Equivocation {
					voter: "Bob".to_string(),
					blocks: vec![1, 2],
				}
			]),
			EquivocationDetected::Precommit(vec![
				Equivocation {
					voter: "Alice".to_string(),
					blocks: vec![1, 2],
				},
				Equivocation {
					voter: "Bob".to_string(),
					blocks: vec![1, 2],
				}
			]),
		],
	);
}

#[test]
fn basic_example_with_prevotes() {
	let mut world = World::new(setup_voters_with_two_finalized_forks(
		Behaviour::ReturnPrevotes,
	));

	world.list_commits();

	println!("\n*** Starting loop ***\n");

	while !world.completed() {
		let requests = world.process_actions();
		let responses = world.handle_requests(requests);
		world.handle_responses(responses);
		world.tick();
	}

	assert_eq!(
		world.equivocations_detected(),
		&[EquivocationDetected::Prevote(vec![
			Equivocation {
				voter: "Alice".to_string(),
				blocks: vec![1, 4],
			},
			Equivocation {
				voter: "Bob".to_string(),
				blocks: vec![1, 4],
			}
		]),],
	);
}
