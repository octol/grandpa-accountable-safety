use crate::{block::BlockNumber, voting::Commit, Request, VoterId};

#[derive(Debug, Clone)]
pub enum Action {
	BroadcastCommits,
	SendBlock(VoterId, BlockNumber),
	RequeueRequest((VoterId, Request)),
}
