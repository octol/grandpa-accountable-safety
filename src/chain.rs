use std::collections::HashMap;

use crate::block::{Block, BlockNumber};

#[derive(Debug)]
pub struct Chain {
	head: BlockNumber,
	finalized: BlockNumber,
	blocks: HashMap<BlockNumber, Block>,
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
			head: blocks[&0].number,
			finalized: blocks[&0].number,
			blocks,
		}
	}

	pub fn add_block(&mut self, block: Block) {
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

	pub fn finalize_block(&mut self, block: BlockNumber) {
		self.finalized = block;
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

	pub fn height(&self) -> u32 {
		self.block_height(self.head)
	}

	pub fn head(&self) -> &Block {
		self.blocks.get(&self.head).unwrap()
	}
}
