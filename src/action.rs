use crate::block::BlockNumber;
use crate::voting::Commit;

#[derive(Debug)]
pub enum Action {
	BroadcastCommits,
	SendBlock(String, BlockNumber),
}
