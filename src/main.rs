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

use std::collections::HashMap;

type BlockNumber = u32;

#[derive(Debug)]
struct Block {
    number: BlockNumber,
    parent: BlockNumber,
}

type VoterId  = u8;

struct Prevote {
    target_number: BlockNumber,
    id: VoterId,
}

struct Precommit {
    target_number: BlockNumber,
    id: VoterId,
}

struct Commit {
    target_number: BlockNumber,
    precommits: Vec<Precommit>,
}

#[derive(Debug)]
struct Chain {
    head: BlockNumber,
    blocks: HashMap<BlockNumber, Block>,
}

impl Chain {
    fn new() -> Self {
        let mut blocks = HashMap::new();

        let genesis = Block { number: 0, parent: 0 };
        blocks.insert(genesis.number, genesis);

        Self { head: blocks[&0].number, blocks }
    }

    fn add_block(&mut self, block: Block) {
        matches!(self.blocks.insert(block.number, block), None);
    }

    fn head() -> Block {
        todo!();
    }
}

fn main() {
    // Create an example chain, including votes
    let mut chain = Chain::new();
    chain.add_block(Block { number: 1, parent: 0 });
    chain.add_block(Block { number: 2, parent: 1 });
    chain.add_block(Block { number: 3, parent: 2 });
    chain.add_block(Block { number: 4, parent: 3 });

    chain.add_block(Block { number: 5, parent: 1 });
    chain.add_block(Block { number: 6, parent: 5 });
    chain.add_block(Block { number: 7, parent: 6 });

    dbg!(&chain);
}
