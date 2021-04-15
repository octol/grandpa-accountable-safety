use crate::{Request, VoterId, block::BlockNumber, protocol::Query};

pub type TriggerAtTick = usize;

#[derive(Debug, Clone)]
pub enum Action {
	BroadcastCommits,
	SendBlock(VoterId, BlockNumber),
	RequeueRequest((VoterId, Request)),
	AskVotersAboutEstimate(Query),
}
