use crate::VoterId;
use crate::block::BlockNumber;
use crate::voting::Commit;

#[derive(Debug)]
pub enum Action {
	BroadcastCommits,
	SendBlock(VoterId, BlockNumber),
}
