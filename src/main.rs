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
	voting::{VoterSet, VotingRound, VotingRounds},
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
		let mut voters = Vec::new();

		let voter = Voter {
			id: "Alice".to_string(),
		};
		voters.push(voter);
		let voter = Voter {
			id: "Bob".to_string(),
		};
		voters.push(voter);
		let voter = Voter {
			id: "Carol".to_string(),
		};
		voters.push(voter);
		let voter = Voter {
			id: "Dave".to_string(),
		};
		voters.push(voter);

		Self {
			voters,
			current_tick: 0,
		}
	}

	fn tick(&mut self) {
		self.current_tick += 1;
	}

	fn completed(&self) -> bool {
		self.current_tick >= MAX_TICKS
	}
}

fn main() {
	let mut env = Environment::new();

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
