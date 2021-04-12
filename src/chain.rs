use std::collections::HashMap;

use crate::{
	block::{Block, BlockNumber},
	voting::{Commit, RoundNumber},
};

#[derive(Debug, Clone)]
pub struct Chain {
	blocks: HashMap<BlockNumber, Block>,
	commits: HashMap<BlockNumber, Commit>,
	finalized_rounds: HashMap<BlockNumber, RoundNumber>,
}

impl Chain {
	pub fn new() -> Self {
		let mut blocks = HashMap::new();
		let genesis = Block {
			number: 0,
			parent: 0,
		};
		blocks.insert(genesis.number, genesis);
		Self {
			blocks,
			commits: Default::default(),
			finalized_rounds: Default::default(),
		}
	}

	pub fn new_from(blocks: &[(BlockNumber, BlockNumber)]) -> Self {
		let mut chain = Chain::new();

		for b in blocks {
			chain.add_block(Block::new(b.0, b.1));
		}

		chain
	}

	pub fn add_block(&mut self, block: Block) {
		// Check that parent exists
		assert!(matches!(self.blocks.get(&block.parent), Some(_)));
		assert!(matches!(
			self.blocks.insert(block.number, block.clone()),
			None
		));
	}

	pub fn finalize_block(
		&mut self,
		block: BlockNumber,
		round_number: RoundNumber,
		commit: Commit,
	) {
		// self.last_finalized = block;
		assert_eq!(block, commit.target_number);
		assert!(matches!(self.commits.insert(block, commit), None));
		assert!(matches!(
			self.finalized_rounds.insert(block, round_number),
			None
		));
	}

	pub fn block_height(&self, block: BlockNumber) -> u32 {
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

	pub fn commit_for_block(&self, block: BlockNumber) -> Option<&Commit> {
		self.commits.get(&block)
	}

	pub fn commits(&self) -> &HashMap<BlockNumber, Commit> {
		&self.commits
	}

	pub fn is_descendent(&self, block: BlockNumber, ancestor: BlockNumber) -> bool {
		const MAX_BLOCK_LENGTH: u32 = 10000;
		let mut length = 0;

		let mut block = self.blocks.get(&block).unwrap();
		while !block.is_genesis() && length < MAX_BLOCK_LENGTH {
			if block.parent == ancestor {
				return true;
			}
			block = self.blocks.get(&block.parent).unwrap();
			length += 1;
		}
		false
	}

	/// Returns true if the chain leading up to `ancestor` is included in the chain leading up to
	/// `block`. That is, if `block` is a descendant of `ancestor` or the same block.
	pub fn block_includes(&self, block: BlockNumber, ancestor: BlockNumber) -> bool {
		block == ancestor || self.is_descendent(block, ancestor)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn create_test_chain() -> Chain {
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

	#[test]
	fn block_height() {
		let mut chain = Chain::new();
		chain.add_block(Block::new(1, 0));
		chain.add_block(Block::new(2, 1));
		chain.add_block(Block::new(3, 2));
		chain.add_block(Block::new(4, 3));

		assert_eq!(chain.block_height(4), 4);
	}

	#[test]
	fn fork_updates_head() {
		let mut chain = Chain::new();
		chain.add_block(Block::new(1, 0));
		chain.add_block(Block::new(2, 1));
		chain.add_block(Block::new(3, 2));
		chain.add_block(Block::new(4, 3));

		chain.add_block(Block::new(5, 1));
		chain.add_block(Block::new(6, 5));
		chain.add_block(Block::new(7, 6));
		chain.add_block(Block::new(8, 7));

		assert_eq!(chain.block_height(8), 5);
	}

	#[test]
	fn is_ancestor() {
		let chain = create_test_chain();

		assert!(!chain.is_descendent(1, 1));
		assert!(!chain.is_descendent(2, 2));
		assert!(!chain.is_descendent(3, 3));
		assert!(!chain.is_descendent(4, 4));

		assert!(chain.block_includes(1, 1));
		assert!(chain.block_includes(2, 2));
		assert!(chain.block_includes(3, 3));
		assert!(chain.block_includes(4, 4));

		assert!(chain.is_descendent(2, 1));
		assert!(chain.is_descendent(3, 1));
		assert!(chain.is_descendent(4, 1));

		assert!(chain.block_includes(2, 1));
		assert!(chain.block_includes(3, 1));
		assert!(chain.block_includes(4, 1));

		assert!(!chain.is_descendent(2, 5));
		assert!(!chain.is_descendent(3, 5));
		assert!(!chain.is_descendent(4, 5));

		assert!(chain.is_descendent(5, 1));
		assert!(chain.is_descendent(6, 1));
		assert!(chain.is_descendent(7, 1));
		assert!(chain.is_descendent(8, 1));

		assert!(!chain.is_descendent(5, 2));
		assert!(!chain.is_descendent(6, 2));
		assert!(!chain.is_descendent(7, 2));
		assert!(!chain.is_descendent(8, 2));
	}
}
