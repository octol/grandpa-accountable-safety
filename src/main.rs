// Accountable Safety for GRANDPA
// ==============================
//
// Accountable Safety for GRANDPA is a synchronous interactive protocol for tracking down and
// proving after the fact when participants misbehave. The idea is that even is more than 1/3 of
// participants misbehave and finalize conflicting forks, they will not get away with and will get
// their stake slashed.
//
// In the GRANDPA paper[1] there is a proof by construction for showing that if two blocks B and B'
// for which valid commit messages were sent, but do not lie on the same chain, then there are at
// least f + 1 Byzantine voters. The proof itself then provides the procedure for tracking down this
// set of misbehaving voters.
//
// Definitions
// ===========
//
// We refer to the GRANDPA paper [1] for in-depth material, it is still useful to restate a some of
// the more important definitions here.
//
// GHOST Function
// --------------
// The function g(S) takes the set of votes and returns the block B with the highest block number
// such that S has a supermajority for B.
//
// Estimate
// --------
// E_{r,v} is voter v's estimate of what might have been finalized in round r, given by the last
// block in the chain with head g(V_{r,v}) for which it is possible for C_{r,v} to have a
// supermajority.
//
// Completable
// -----------
// If either E_{r,v} < g(V_{r,v}) or it is impossible for C_{r,v} to have a supermajority for any
// children of g(V_{r,v}), then we say that v sees that round r as completable.
//
// In other words, when E_{r,v} contains everything that could have been finalized in round r.
//
// E_{r,v} having supermajority means that E_{r,v} < g(V_{r,v}).
// WIP(JON): how?
//
// Outline of the Procedure
// ========================
//
// Step 0.
// -------
//
// The first step is detecting blocks B and B' on two different branches being finalized.
// We assume B' was finalized in a later round r' than B, which was finalized in round r.
// That is, r'> r.
//
// o-o-o-B
//    \o-o-B'
//
// Step 1. start asking questions about B'
// ---------------------------------------
//
// Q: Why the estimate did not include B when prevoting for B'
// A: A set S of prevotes or a set S of precommits of the preceding round.
//    In either case such that it is impossible for S to have a supermajority for B.
//
// (Repeat for each round back to round r+1.)
//
// Step 2. reach the round after which B was finalized
// ---------------------------------------------------
//
// The reply for round r+1 will contain a set S of either prevotes or precommits
// - If precommits: take union with precommits in commit msg for B to find equivocators.
// - If prevotes: ask the precommitters for B.
//
// Step 3. instead ask the precommitters for B
// -------------------------------------------
//
// Q: Ask all precommitters in the in commit msg for B, which prevotes have you seen?
// A: A set T of prevotes with a supermajority for B.
//    Take the union with S and find the equivocators.
//
// Example
// =======
//
// Consider the set of voters V = {a, b, c, d} and the set of blocks
//
// 	0 -> 1 -> 2 -> 3 -> 4
//        \-> 5 -> 6 -> 7 -> 8
//
// which has two heads. We now lay out the 3 grandpa rounds for this, where 2 out of the 4 voters
// manage to partition the voting set long enough to finalize blocks on both forks.
//
// - Round 0: genesis.
//
// - Round 1: vote on each side of the fork and the finalize the common ancestor of both.
//
// 	V_1 = (2, 2, 5, 5)
//  C_1 = (1, 1, 1, 1)
//
// Broadcast commit message for finalizing block 1 in round 1.
//
// - Round 2:
//
// We assume that the set of voters are partitioned into two sets:
//
// 	{a, b, c}
//	{a, b, d}
//
// with a and b in the overlapping set, meaning that c and d are not communicating. This allows a
// and b to control the voting by presenting different votes to c and d.
// The first group finalizes the first fork
//
// 	V_{2,1} = (4, 4, 2, _)
//  C_{2,1} = (2, 2, 2, _)
//
// and broadcasts a commit message for finalizing block 2 in round 2.
// The second group does not finalize anything
//
// 	V_{2,2} = (1, 1, _, 5)
//  C_{2,2} = (1, 1, _, 1)
//
// - Round 3:
//
// The first group does not finalize anything
//
// 	V_{3,1} = (4, 4, 2, _)
//  C_{3,1} = (2, 2, 2, _)
//
// The second group finalizes the second fork
//
// 	V_{3,2} = (8, 8, _, 8)
//  C_{3,2} = (8, 8, _, 8)
//
// and broadcasts a commit message for finalizing block 8 in round 3.
//
// After these rounds we now have a situation where we sent valid commit messages finalizing blocks
// on both forks of the chain. We now illustrate the steps needed to uncover the equivocating voters.
//
// - Step 0: detect that block 2 and 8 on different branches are finalized.
//
//  We receive commits for both finalized blocks, and see that one is not the a common ancestor of
//  the other.
//
// - Step 1: (not applicable since we are at round r+1 already)
//
// - Step 2:
//  Q: Why did the estimate for round 2 in round 3 not include block 2 when prevoting or
//     precommitting for 8?
//
// (NOTE: we are only asking the voters that precomitted for 8, so (a, b, _, d)).
//
// Alternative 1:
//
//  A: A set of precommits for round 2, that shows it's impossible to have supermajority for
//     block 2 in round 2.
//
// Responses
//  S_a = {1, 1, _, 1}
//  S_b = {1, 1, _, 1}
//  S_d = {1, 1, _, 1}
//
// (NOTE: "a" and "b" chooses to not send the precommits it saw as part of group 1 as that would
// not have been a valid reply.)
//
// Take union with precommits in commit message for block 2 to find equivocators.
// 	{4, 4, 4, _} U {1, 1, _, 1} => a and b appears twice, they *equivocated*!
//
// Alternative 2:
// (QUESTION: what is the point of even accepting prevotes as reply to the query?)
//  A: A set of prevotes for round 2.
//
//  S_a = {1, 1, _, 5}
//  S_b = {1, 1, _, 5}
//  S_d = {1, 1, _, 5}
//
// Step 3.
//  Q: Ask precommitters in commit message for block 2 who voted for blocks in the 2 fork, which
//     prevotes have you seen?
//  A: This is voters {a, b, c, _}
//
//  T_a = {4, 4, 2, _}
//  T_b = {4, 4, 2, _}
//  T_c = {4, 4, 2, _}
//
// Take the union S U T
//
//  (1, 1, _, 5) U (4, 4, 2, _)  => a and b occurs twice and *equivocated*.
//
// References
// ==========
//
// [1]: https://github.com/w3f/consensus/blob/master/pdf/grandpa.pdf,
//      https://arxiv.org/pdf/2007.01560.pdf

use block::BlockNumber;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use voting::VoterId;

use crate::block::Block;
use crate::chain::Chain;
use crate::voting::{Commit, Precommit, Prevote, VoterSet, VotingRound, VotingRounds};

mod block;
mod chain;
mod voting;

fn main() {
	run_chain_scenario_from_paper();
}

// The idea in the scenario is that we will get conflicting results from the commit message and the
// set of precommits returned when querying the voters. This allows us to identify the
// equivocators.
fn run_chain_scenario_from_paper() {
	let (chain, voting_rounds) = create_chain_with_two_forks_and_equivocations();

	let last_finalized_round = 3;

	// Query voter(s)
	//
	// Step 0: detect that block 2 and 8 on different branches are finalized.
	//
	//  We receive commits for both finalized blocks, commit2_1 and commit3_2, and see that one is
	//  not the a common ancestor of the other.
	assert!(!chain.is_descendent(2, 8));
	assert!(!chain.is_descendent(8, 2));

	// Step 1: (not applicable since we are at round r+1 already)

	// ... update this example with one more round so that this step is included ...

	// Step 2:
	//  Q: Why did the estimate for round 2 in round 3 not include block 2 when prevoting or
	//     precommitting for block 8?
	//
	// (NOTE: we are only asking the voters that precommitted for 8, so {a, b, _, d}).
	// (what about prevoted?)
	//
	// Alternative 1:
	//  A: A set of precommits for round 2, that shows it's impossible to have supermajority for
	//     block 2 in round 2.
	let round2_2 = voting_rounds.get(&2).unwrap()[1].clone();
	let response_is_precommits = round2_2.precommits.clone();
	validate_precommit_reply(&response_is_precommits, 2, &round2_2.voter_set, &chain);

	cross_check_precommit_reply_against_commit(
		&response_is_precommits,
		chain.commit_for_block(2).unwrap().clone(),
	);

	// Alternative 2:
	//  A: A set of prevotes for round 2.
	//  (QUESTION: what is the point of even accepting prevotes as reply to the query?)

	let response_is_prevotes = round2_2.prevotes;

	// Step 3.
	//  Q: Ask precommitters in commit message for block 2 who voted for blocks in the 2 fork, which
	//     prevotes have you seen?

	let voters_in_commit = chain
		.commit_for_block(2)
		.unwrap()
		.precommits
		.iter()
		.map(|p| p.id)
		.collect::<Vec<_>>();

	// ... ask `voters_in_commit` what prevotes they have seen ...

	let round2_1 = voting_rounds.get(&2).unwrap()[0].clone();
	let followup_response_in_prevotes = round2_1.prevotes;

	cross_check_prevote_reply_against_prevotes_seen(
		response_is_prevotes,
		followup_response_in_prevotes,
	);
}

// Create a chain with two forks
//  0 -> 1 -> 2 -> 3 -> 4
//        \-> 5 -> 6 -> 7 -> 8
fn create_chain_with_two_forks() -> Chain {
	let mut chain = Chain::new();
	chain.add_block(Block::new(1, 0));

	// First fork
	chain.add_block(Block::new(2, 1));
	chain.add_block(Block::new(3, 2));
	chain.add_block(Block::new(4, 3));

	// Second, longer, fork
	chain.add_block(Block::new(5, 1));
	chain.add_block(Block::new(6, 5));
	chain.add_block(Block::new(7, 6));
	chain.add_block(Block::new(8, 7));

	assert_eq!(chain.block_height(4), 4);
	assert_eq!(chain.block_height(8), 5);
	chain
}

// Create a chain with two forks with both sides being finalized.
// The block 2 on is finalized in round 2, and block 8 is finalized in round 3.
fn create_chain_with_two_forks_and_equivocations() -> (Chain, VotingRounds) {
	let mut chain = create_chain_with_two_forks();
	let voter_set = VoterSet::new(&["a", "b", "c", "d"]);

	let mut voting_rounds = HashMap::new();

	// Round 0: is genesis.

	// Round 1: vote on each side of the fork and the finalize the common ancestor of both.
	// "a" and "b" sees first fork, "c" and "d" seems the second fork.
	{
		let mut round = VotingRound::new(1, voter_set.clone());
		round.prevote(&[(2, "a"), (2, "b"), (5, "c"), (5, "d")]);
		round.precommit(&[(1, "a"), (1, "b"), (1, "c"), (1, "d")]);
		let commit = Commit::new(1, round.precommits.clone());
		chain.finalize_block(1, commit);
		voting_rounds.insert(round.round_number, vec![round]);
	}

	// Round 2:
	// Split into two: ("a", "b", "c") and ("a", "b", "d")
	// The first group "1" finalizes the first fork
	let mut round2_1 = VotingRound::new(2, voter_set.clone());
	round2_1.prevote(&[(4, "a"), (4, "b"), (2, "c")]);
	round2_1.precommit(&[(2, "a"), (2, "b"), (2, "c")]);
	let commit2_1 = Commit::new(2, round2_1.precommits.clone());
	chain.finalize_block(2, commit2_1.clone());

	// The second group "2" does not finalize anything
	let mut round2_2 = VotingRound::new(2, voter_set.clone());
	round2_2.prevote(&[(1, "a"), (1, "b"), (5, "d")]);
	round2_2.precommit(&[(1, "a"), (1, "b"), (1, "d")]);
	voting_rounds.insert(round2_1.round_number, vec![round2_1, round2_2]);

	// Round 3:
	// The first group "1" does not finalize anything
	let mut round3_1 = VotingRound::new(3, voter_set.clone());
	round3_1.prevote(&[(4, "a"), (4, "b"), (2, "c")]);
	round3_1.precommit(&[(2, "a"), (2, "b"), (2, "c")]);

	// The second group "2" finalizes the second fork
	// "d" has not seen the commit from the first group in round 2.
	let mut round3_2 = VotingRound::new(3, voter_set.clone());
	round3_2.prevote(&[(8, "a"), (8, "b"), (8, "d")]);
	round3_2.precommit(&[(8, "a"), (8, "b"), (8, "d")]);
	let commit3_2 = Commit::new(8, round3_1.precommits.clone());
	chain.finalize_block(8, commit3_2);
	voting_rounds.insert(round3_1.round_number, vec![round3_1, round3_2]);

	(chain, voting_rounds)
}

// Check the validity of a response containing precommits.
// The purpose of the response is to return a set of precommits showing it is impossible to have a
// supermajority for the given block.
fn validate_precommit_reply(
	response: &Vec<Precommit>,
	block: BlockNumber,
	voter_set: &VoterSet,
	chain: &Chain,
) {
	// No equivocations
	let unique_voters: HashSet<VoterId> = response.iter().map(|pre| pre.id).unique().collect();
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
	assert!(!(3 * (precommits_includes_block + absent_voters) > 2 * num_voters));
}

// Cross check against precommitters in commit message
fn cross_check_precommit_reply_against_commit(s: &Vec<Precommit>, commit: Commit) {
	for precommit in &commit.precommits {
		let equivocated_votes: Vec<_> = s.iter().filter(|pre| pre.id == precommit.id).collect();

		if !equivocated_votes.is_empty() {
			print!(
				"Precommit equivocation detected by {} for {}",
				precommit.id, precommit.target_number
			);
			equivocated_votes.iter().for_each(|e| {
				print!(", {}", e.target_number);
			});
			print!("\n");
		}
	}
}

fn cross_check_prevote_reply_against_prevotes_seen(s: Vec<Prevote>, t: Vec<Prevote>) {
	for prevote in &t {
		let equivocated_votes: Vec<_> = s.iter().filter(|pre| pre.id == prevote.id).collect();

		if !equivocated_votes.is_empty() {
			print!(
				"Prevote equivocation detected by {} for {}",
				prevote.id, prevote.target_number
			);
			equivocated_votes.iter().for_each(|e| {
				print!(", {}", e.target_number);
			});
			print!("\n");
		}
	}
}

fn is_valid_reply(s: &Vec<Precommit>) -> bool {
	true
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn block_height() {
		let mut chain = Chain::new();
		assert_eq!(chain.head(), &Block::new(0, 0));
		chain.add_block(Block::new(1, 0));
		assert_eq!(chain.head(), &Block::new(1, 0));
		chain.add_block(Block::new(2, 1));
		assert_eq!(chain.head(), &Block::new(2, 1));
		chain.add_block(Block::new(3, 2));
		assert_eq!(chain.head(), &Block::new(3, 2));
		chain.add_block(Block::new(4, 3));
		assert_eq!(chain.head(), &Block::new(4, 3));

		assert_eq!(chain.block_height(4), 4);
		assert_eq!(chain.height(), 4);
	}

	#[test]
	fn fork_updates_head() {
		let mut chain = Chain::new();
		chain.add_block(Block::new(1, 0));
		assert_eq!(chain.head(), &Block::new(1, 0));
		chain.add_block(Block::new(2, 1));
		assert_eq!(chain.head(), &Block::new(2, 1));
		chain.add_block(Block::new(3, 2));
		assert_eq!(chain.head(), &Block::new(3, 2));
		chain.add_block(Block::new(4, 3));
		assert_eq!(chain.head(), &Block::new(4, 3));

		chain.add_block(Block::new(5, 1));
		assert_eq!(chain.head(), &Block::new(4, 3));
		chain.add_block(Block::new(6, 5));
		assert_eq!(chain.head(), &Block::new(4, 3));
		chain.add_block(Block::new(7, 6));
		assert_eq!(chain.head(), &Block::new(4, 3));
		chain.add_block(Block::new(8, 7));
		assert_eq!(chain.head(), &Block::new(8, 7));

		assert_eq!(chain.height(), 5);
	}
}
