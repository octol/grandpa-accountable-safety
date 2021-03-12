pub type BlockNumber = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub number: BlockNumber,
    pub parent: BlockNumber,
}

impl Block {
    pub fn new(number: BlockNumber, parent: BlockNumber) -> Self {
        Self { number, parent }
    }
}
