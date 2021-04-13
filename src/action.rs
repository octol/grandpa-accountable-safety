use crate::block::BlockNumber;
use crate::voting::Commit;
use crate::Request;
use crate::VoterId;

#[derive(Debug)]
pub enum Action {
	BroadcastCommits,
	SendBlock(VoterId, BlockNumber),
	RequeueRequest((VoterId, Request)),
}
