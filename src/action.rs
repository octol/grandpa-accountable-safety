use crate::{block::BlockNumber, Request, VoterId};

#[derive(Debug, Clone)]
pub enum Action {
	BroadcastCommits,
	SendBlock(VoterId, BlockNumber),
	RequeueRequest((VoterId, Request)),
}
