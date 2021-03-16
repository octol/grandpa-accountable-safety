// Outline of Accountable safety algorithm
// =======================================
//
// Step 0.
// -------
//
// Detect blocks B and B' on two different branches finalized
// Assume B' was finalized in a later round than B, r'> r.
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
// The reply for round r+1 will contain a set S of either prevotes or precommites
// - If precommits: take union with precommits in commit msg for B to find equivocators.
// - If prevotes: ask the precommitters for B.
//
// Step 3. instead ask the precommitters for B
// -------------------------------------------
//
// Q: Ask all precommitters in the in commit msg for B, which prevotes have you seen?
// A: A set T of prevotes with a supermajority for B.
//    Take the union with S and find the equivocators.

// Definitions
// ===========

// GHOST Function
// --------------
// The function g(S) takes the set of votes and returns the block B with the highest block number
// such that S has a supermajority for B.

// Estimate
// --------
// E_{r,v} is v's estimate of what might have been finalized in round r, given by the last block in
// the chain with head g(V_{r,v}) for which it is possible for C_{r,v} to have a supermajority.

// Completable
// -----------
// If either E_{r,v} < g(V_{r,v}) or it is impossible for C_{r,v} to have a supermajority for any
// children of g(V_{r,v}), then we say that v sees that round r as completable.
//
// In other words, when E_{r,v} contains everything that could have been finalized in round r.

// E_{r,v} having supermajority => E_{r,v} < g(V_{r,v}).

use crate::block::Block;
use crate::chain::Chain;
use crate::voting::{VoterSet, VotingRound};

mod block;
mod chain;
mod voting;

fn main() {
    safe_chain();
    unsafe_chain();
}

fn safe_chain() {
    let mut chain = create_chain();
    let voter_set = VoterSet::new(&["a", "b", "c", "d"]);

    // Round 0: is genesis.

    // Round 1: Round starts when the previous round is completable.
    let mut round1 = VotingRound::new(1, voter_set.clone());
    // Prevote for the head of the best chain containing E_0
    round1.prevote(&[(2, "a"), (2, "b"), (5, "c"), (5, "d")]);
    // Wait for g(V) >= E_0 = 0
    // g(V) = 1
    round1.precommit(&[(1, "a"), (1, "b"), (1, "c"), (1, "d")]);
    // g(C) = 1
    // Broadcast commit for B = g(C) = 1
    chain.finalize_block(1);

    // Round 2
    let mut round2 = VotingRound::new(2, voter_set.clone());
    round2.prevote(&[(4, "a"), (8, "b"), (8, "c"), (8, "d")]);
    round2.precommit(&[(8, "a"), (8, "b"), (8, "c"), (8, "d")]);
    chain.finalize_block(8);

    // Query voter(s)
}

// The idea in the scenario is that we will get conflicting results from the commit message and the
// set of precommits returned when querying the voters. This allows us to identify the
// equivocators.
fn unsafe_chain_scenario_from_paper() {
    let mut chain = create_chain();
    let voter_set = VoterSet::new(&["a", "b", "c", "d"]);

    // Round 0: is genesis.

    // Round 1: vote on each side of the fork and the finalize the common ancestor of both.
    // "a" and "b" sees first fork, "c" and "d" seems the second fork.
    let mut round1 = VotingRound::new(1, voter_set.clone());
    round1.prevote(&[(2, "a"), (2, "b"), (5, "c"), (5, "d")]);
    round1.precommit(&[(1, "a"), (1, "b"), (1, "c"), (1, "d")]);
    chain.finalize_block(1);

    // Round 2: finalize on first fork
    // "a" and "b" see the same block 2 and the first fork as the longest. "c" and "d" now see
    // block 3, meaning that they also now see the first fork as the longest.
    let mut round2 = VotingRound::new(2, voter_set.clone());
    round2.prevote(&[(2, "a"), (2, "b"), (3, "c"), (3, "d")]);
    round2.precommit(&[(2, "a"), (2, "b"), (2, "c"), (2, "d")]);
    chain.finalize_block(2);

    // Round 3: finalize on second fork
    // "a" sees another block on the same fork, "b", "c", "d" all see block 8 on the second fork as
    // the longest. The first fork has been finalized however, invalidating block 8.
    let mut round3 = VotingRound::new(3, voter_set.clone());
    round3.prevote(&[(3, "a"), (8, "b"), (8, "c"), (8, "d")]);
    round3.precommit(&[(8, "a"), (8, "b"), (8, "c"), (8, "d")]);
    chain.finalize_block(8);

    // Query voter(s)
    // Step 0: detect that block 2 and 8 on different branches are finalized.
    // Step 1: (not applicable since we are at round r+1 already)
    // Step 2:
    //  Q: Why did the estimate for round 2 in round 3 not include block 2 when prevoting or
    //     precommitting for 8?
    //
    // Alternative 1:
    //  A: A set of precommits for round 2, that shows it's impossible to have supermajority for
    //     block 2 in round 2.
    //
    //  S_a = {2, 2, 2, 2}
    //  S_b = {2, 2, 2, 2}
    //  S_c = {2, 2, 5, 5}
    //  S_d = {2, 2, 5, 5}
    //
    //  NOTE: "c" and "d" must collude here to sign each others precommitts that are not part of
    //  the commit message.
    //
    //  => Take union with precommits in commit message for block 2 to find equivocators.
    //
    //  {2, 2, 5, 5} U {2, 2, 2, 2}
    //  => "c" and "d" appears twice. They equivocated.
    //  But where does this set S with "false" votes come from?
    //  And presumably this is signed somehow so that we can trust the authenticity.
    //
    // Alternative 2:
    //  A: A set of prevotes for round 2.
    //
    //  S = {2, 2, 5, 5}
    //
    // Step 3.
    //  Q: Ask precommitters in commit message for block 2 which prevotes have you seen?
    //  A:
    //
    //  S_a = {2, 2, 3, 3}
    //  S_b = {2, 2, 3, 3}
    //  S_c = {2, 2, 3, 3}
    //  S_d = {2, 2, 3, 3}
    //
    //  {2, 2, 3, 3} U {2, 2, 5, 5}
    //  => "c" and "d" occuts twice and equivocated
    //
}

// Here the byzantine actors do return the honest precommits, and that shows they just ignored the
// finalized block.
fn unsafe_chain() {
    let mut chain = create_chain();
    let voter_set = VoterSet::new(&["a", "b", "c", "d"]);

    // Round 0: is genesis.

    // Round 1: vote on each side of the fork and the finalize the common ancestor of both.
    // "a" and "b" sees first fork, "c" and "d" seems the second fork.
    let mut round1 = VotingRound::new(1, voter_set.clone());
    round1.prevote(&[(2, "a"), (2, "b"), (5, "c"), (5, "d")]);
    round1.precommit(&[(1, "a"), (1, "b"), (1, "c"), (1, "d")]);
    chain.finalize_block(1);

    // Round 2: finalize on first fork
    // "a" and "b" see the same block 2 and the first fork as the longest. "c" and "d" now see
    // block 3, meaning that they also now see the first fork as the longest.
    let mut round2 = VotingRound::new(2, voter_set.clone());
    round2.prevote(&[(2, "a"), (2, "b"), (3, "c"), (3, "d")]);
    round2.precommit(&[(2, "a"), (2, "b"), (2, "c"), (2, "d")]);
    chain.finalize_block(2);

    // Round 3: finalize on second fork
    // "a" sees another block on the same fork, "b", "c", "d" all see block 8 on the second fork as
    // the longest. The first fork has been finalized however, invalidating block 8.
    let mut round3 = VotingRound::new(3, voter_set.clone());
    round3.prevote(&[(3, "a"), (8, "b"), (8, "c"), (8, "d")]);
    round3.precommit(&[(8, "a"), (8, "b"), (8, "c"), (8, "d")]);
    chain.finalize_block(8);

    // Query voter(s)
    // Step 0: block 2 and 8 on different branches are finalized.
    // Step 1: (not applicable since we are at round r+1 already)
    // Step 2:
    //  Q: Why did the estimate for round 2 in round 3 not include block 2 when prevoting or
    //     precommitting for 8?
    //
    // Alternative 1:
    //  A: A set of precommits for round 2, S = {2, 2, 2, 2}
    //  => Take union with precommits in commit msg for block 2 to find equivocators.
    //
    //  {2, 2, 2, 2} U {2, 2, 2, 2}
    //  => ?? (finished)
    //  => What does this mean?
    //  This to me looks a like failure to respond in valid way. S did infact have supermajority
    //  for block 2.
    //
    //  We should here see that "b", "c", "d" all had estimates that did not include 2.
    //
    // Alternative 2:
    //  A: A set of prevotes for round 2.
    //
    // Step 3.
    //  Q: Ask precommitters in commit msg for block 2 which prevotes have you seen?
    //  A: {2, 2, 3, 3}
    //
    //  {2, 2, 3, 3} U {2, 2, 3, 3}
    //  => ??
    //
}

fn create_chain() -> Chain {
    // 0 -> 1 -> 2 -> 3 -> 4
    //       \-> 5 -> 6 -> 7 -> 8
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
