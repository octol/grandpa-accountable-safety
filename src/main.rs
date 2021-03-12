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

use crate::block::Block;
use crate::chain::Chain;
use crate::voting::{VoterSet, VotingRound};

mod block;
mod chain;
mod voting;

fn main() {
    let mut chain = create_chain();
    let voter_set = VoterSet::new(&["a", "b", "c", "d"]);

    // Round 1
    let mut round1 = VotingRound::new(1, voter_set.clone());
    // Prevote for the head of the best chain containing E_0
    round1.prevote(&[(2, "a"), (2, "b"), (5, "c"), (5, "d")]);
    // Wait for g(V) >= E_0
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
