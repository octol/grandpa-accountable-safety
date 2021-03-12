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

use std::collections::{HashMap, HashSet};

type BlockNumber = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Block {
    number: BlockNumber,
    parent: BlockNumber,
}

type VoterId = u8;

struct VoterSet {
    voters: HashSet<VoterId>,
}

impl VoterSet {
    fn new(voter_ids: &[VoterId]) -> Self {
        Self {
            voters: voter_ids.iter().cloned().collect(),
        }
    }
}

struct VotingRound {
    round_number: u64,
    prevotes: Vec<Prevote>,
    precommits: Vec<Precommit>,
}

impl VotingRound {
    fn new(round_number: u64) -> Self {
        Self {
            round_number,
            prevotes: Default::default(),
            precommits: Default::default(),
        }
    }
}

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

        let genesis = Block {
            number: 0,
            parent: 0,
        };
        blocks.insert(genesis.number, genesis);

        Self {
            head: blocks[&0].number,
            blocks,
        }
    }

    fn add_block(&mut self, block: Block) {
        // Check that parent exists
        assert!(matches!(self.blocks.get(&block.parent), Some(_)));
        assert!(matches!(
            self.blocks.insert(block.number, block.clone()),
            None
        ));

        // Update head if the new block has a height height
        if self.block_height(block.number) > self.head {
            self.head = block.number;
        }
    }

    fn block_height(&self, block: BlockNumber) -> u32 {
        let mut block = self.blocks.get(&block).unwrap();
        let mut height = 0;

        const MAX_HEIGHT: u32 = 10000;
        while block.number > 0 && height < MAX_HEIGHT {
            block = self.blocks.get(&block.parent).unwrap();
            height += 1;
        }
        assert!(height < MAX_HEIGHT, "Maybe a loop");
        height
    }

    fn height(&self) -> u32 {
        self.block_height(self.head)
    }

    fn head(&self) -> &Block {
        self.blocks.get(&self.head).unwrap()
    }
}

fn main() {
    // Create an example chain, including votes
    let chain = {
        let mut chain = Chain::new();
        chain.add_block(Block {
            number: 1,
            parent: 0,
        });
        chain.add_block(Block {
            number: 2,
            parent: 1,
        });
        chain.add_block(Block {
            number: 3,
            parent: 2,
        });
        chain
    };

    let voter_set = VoterSet::new(&[0, 1, 2]);

    // Round 1
    let mut round1 = VotingRound::new(1);
    // Prevote for the head of the best chain containing E_0
    round1.prevotes = vec![
        Prevote {
            target_number: 1,
            id: 0,
        },
        Prevote {
            target_number: 1,
            id: 1,
        },
        Prevote {
            target_number: 2,
            id: 2,
        },
    ]
    .into_iter()
    .collect();
    // Wait for g(V) >= E_0
    // g(V) = 1
    let c0 = Precommit {
        target_number: 1,
        id: 0,
    };
    let c1 = Precommit {
        target_number: 1,
        id: 1,
    };
    let c2 = Precommit {
        target_number: 1,
        id: 2,
    };
    round1.precommits = vec![c0, c1, c2].into_iter().collect();
    // g(C) = 1
    // Broadcast commit for B = g(C) = 1
    //chain.finalize(1);

    // Round 2

    // Query voter(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_height() {
        let mut chain = Chain::new();
        assert_eq!(
            chain.head(),
            &Block {
                number: 0,
                parent: 0
            }
        );
        chain.add_block(Block {
            number: 1,
            parent: 0,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 1,
                parent: 0
            }
        );
        chain.add_block(Block {
            number: 2,
            parent: 1,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 2,
                parent: 1
            }
        );
        chain.add_block(Block {
            number: 3,
            parent: 2,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 3,
                parent: 2
            }
        );
        chain.add_block(Block {
            number: 4,
            parent: 3,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 4,
                parent: 3
            }
        );

        assert_eq!(chain.block_height(4), 4);
        assert_eq!(chain.height(), 4);
    }

    #[test]
    fn fork_updates_head() {
        let mut chain = Chain::new();
        chain.add_block(Block {
            number: 1,
            parent: 0,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 1,
                parent: 0
            }
        );
        chain.add_block(Block {
            number: 2,
            parent: 1,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 2,
                parent: 1
            }
        );
        chain.add_block(Block {
            number: 3,
            parent: 2,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 3,
                parent: 2
            }
        );
        chain.add_block(Block {
            number: 4,
            parent: 3,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 4,
                parent: 3
            }
        );

        chain.add_block(Block {
            number: 5,
            parent: 1,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 4,
                parent: 3
            }
        );
        chain.add_block(Block {
            number: 6,
            parent: 5,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 4,
                parent: 3
            }
        );
        chain.add_block(Block {
            number: 7,
            parent: 6,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 4,
                parent: 3
            }
        );
        chain.add_block(Block {
            number: 8,
            parent: 7,
        });
        assert_eq!(
            chain.head(),
            &Block {
                number: 8,
                parent: 7
            }
        );

        assert_eq!(chain.height(), 5);
    }
}
