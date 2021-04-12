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
// References
// ==========
//
// [1]: https://github.com/w3f/consensus/blob/master/pdf/grandpa.pdf,
//      https://arxiv.org/pdf/2007.01560.pdf

use crate::{
	chain::Chain,
	voter::Voter,
	voting::{VoterSet, VotingRound, VotingRounds, Commit},
};

mod block;
mod chain;
mod example;
mod voter;
mod voting;

const MAX_TICKS: usize = 100;

struct Environment {
	voters: Vec<Voter>,
	current_tick: usize,
}

impl Environment {
	fn new() -> Self {
		let names = &["Alice", "Bob", "Carol", "Dave"];
		let voter_set = VoterSet::new(names);

		let mut voters = Vec::new();

		let chain_common = [(1,0)];
		let chain_a_fork = [(2,1), (3,2), (4,3)];
		let chain_b_fork = [(5,1), (6,5), (7,6), (8, 7)];
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

		{
			let mut chain = Chain::new_from(&chain_all);
			let mut voting_rounds = create_common_voting_rounds(&voter_set, &mut chain);
			append_voting_rounds_a(&mut voting_rounds, &voter_set, &mut chain);
			append_voting_rounds_b(&mut voting_rounds, &voter_set, &mut chain);
			voters.push(Voter::new(names[0], chain.clone(), voting_rounds));
		}
		{
			let mut chain = Chain::new_from(&chain_all);
			let mut voting_rounds = create_common_voting_rounds(&voter_set, &mut chain);
			append_voting_rounds_a(&mut voting_rounds, &voter_set, &mut chain);
			append_voting_rounds_b(&mut voting_rounds, &voter_set, &mut chain);
			voters.push(Voter::new(names[1], chain.clone(), voting_rounds));
		}
		{
			let mut chain = Chain::new_from(&chain_a);
			let mut voting_rounds = create_common_voting_rounds(&voter_set, &mut chain);
			append_voting_rounds_a(&mut voting_rounds, &voter_set, &mut chain);
			voters.push(Voter::new(names[2], chain, voting_rounds));
		}
		{
			let mut chain = Chain::new_from(&chain_b);
			let mut voting_rounds = create_common_voting_rounds(&voter_set, &mut chain);
			append_voting_rounds_b(&mut voting_rounds, &voter_set, &mut chain);
			voters.push(Voter::new(names[3], chain, voting_rounds));
		}

		Self {
			voters,
			current_tick: 0,
		}
	}

	fn list_commits(&self) {
		for voter in &self.voters {
			println!("{}:", voter);
			voter.list_commits();
		}
	}

	fn tick(&mut self) {
		self.current_tick += 1;
	}

	fn completed(&self) -> bool {
		self.current_tick >= MAX_TICKS
	}
}

fn create_common_voting_rounds(voter_set: &VoterSet, chain: &mut Chain) -> VotingRounds {
	let mut voting_rounds = VotingRounds::new();
	let voting_round_tag = 0;

	{
		let mut round = VotingRound::new_with_tag(1, voter_set.clone(), voting_round_tag);
		round.prevote(&[(2, "Alice"), (2, "Bob"), (1, "Carol"), (1, "Dave")]);
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
	chain: &mut Chain
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
	chain: &mut Chain
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

fn main() {
	let mut env = Environment::new();

	env.list_commits();

	while !env.completed() {
		// In a game loop we typically have:
		// - check input
		// - update
		// - render
		//
		// In our case maybe it can be something like
		//
		// 1. Process inputs
		//
		// for voter in voters {
		//     voter.act(); // this  can also be to respond
		// }
		//
		// 2.

		env.tick();
	}
}
