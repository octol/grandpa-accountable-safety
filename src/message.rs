use crate::{
	block::{BlockNumber, Block},
	voting::{VoterId, Commit},
};

#[derive(Debug, Clone)]
pub enum Request {
	SendCommit(Commit),
	SendBlocks(Vec<Block>),
}

#[derive(Debug, Clone)]
pub enum Response {
	RequestBlock(BlockNumber),
}

#[derive(Debug)]
pub enum Payload {
	Request(Request),
	Response(Response),
}

impl Payload {
	pub fn request(&self) -> &Request {
		match self {
			Payload::Request(request) => request,
			Payload::Response(..) => panic!("logic error"),
		}
	}

	pub fn response(&self) -> &Response {
		match self {
			Payload::Request(..) => panic!("logic error"),
			Payload::Response(response) => response,
		}
	}
}

#[derive(Debug)]
pub struct Message {
	pub sender: VoterId,
	pub receiver: VoterId,
	pub content: Payload,
}

