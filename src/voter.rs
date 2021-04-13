use crate::{
	action::Action,
	block::{Block, BlockNumber},
	chain::Chain,
	voting::{Commit, VoterSet, VotingRounds},
	VoterId, VoterName,
};
use std::{
	collections::{BTreeMap, HashMap, VecDeque},
	fmt::Display,
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

pub struct Voter {
	pub id: VoterId,
	pub chain: Chain,
	pub voter_set: VoterSet,
	pub voting_rounds: VotingRounds,
	pub actions: Vec<(usize, Action)>,
}

impl Voter {
	pub fn new(
		id: VoterId,
		chain: Chain,
		voter_set: VoterSet,
		voting_rounds: VotingRounds,
	) -> Self {
		Self {
			id,
			chain,
			voter_set,
			voting_rounds,
			actions: Default::default(),
		}
	}

	pub fn add_actions(&mut self, actions: Vec<(usize, Action)>) {
		for (tick, action) in actions {
			self.actions.push((tick, action));
		}
	}

	pub fn list_commits(&self) {
		for c in self.chain.commits() {
			println!("{}", &c.1);
		}
	}

	pub fn commits(&self) -> &HashMap<u32, Commit> {
		self.chain.commits()
	}

	pub fn process_actions(&mut self, current_tick: usize) -> Vec<Message> {
		let actions = self
			.actions
			.iter()
			.filter(|a| a.0 <= current_tick)
			.cloned()
			.collect::<Vec<_>>();
		self.actions.retain(|a| a.0 > current_tick);

		let mut messages = Vec::new();
		for (trigger_time, ref action) in actions {
			match action {
				Action::BroadcastCommits => {
					println!("{}: broadcasting all our commits to all voters", self.id);
					for voter in &self.voter_set.voters {
						if *voter != self.id {
							for (_, c) in self.commits() {
								messages.push(Message {
									sender: self.id.clone(),
									receiver: voter.to_string(),
									content: Payload::Request(Request::SendCommit(c.clone())),
								});
							}
						}
					}
				}
				Action::SendBlock(id, block_number) => {
					println!("{}: send block {} to {}", self.id, block_number, id);
					let blocks = self.chain.get_chain_of_blocks(*block_number);
					if !blocks.is_empty() {
						messages.push(Message {
							sender: self.id.clone(),
							receiver: id.clone(),
							content: Payload::Request(Request::SendBlocks(blocks)),
						});
					} else {
						println!(
							"{}: failed to send block {} as it's not in our chain",
							self.id, block_number
						);
					}
				}
				Action::RequeueRequest((sender, request)) => {
					let should_queue_up = match request {
						Request::SendCommit(commit) => {
							self.chain.knows_about_block(commit.target_number)
						}
						_ => true,
					};
					if should_queue_up {
						messages.push(Message {
							sender: sender.clone(),
							receiver: self.id.clone(),
							content: Payload::Request(request.clone()),
						});
					} else {
						// Postpone
						self.actions.push((trigger_time + 10, action.clone()));
					}
				}
				_ => todo!(),
			}
		}
		messages
	}

	pub fn handle_request(
		&mut self,
		request: (VoterId, Request),
		current_tick: usize,
	) -> Vec<(VoterId, Response)> {
		match request.1 {
			Request::SendCommit(ref commit) => {
				// Ignore commits we already know about
				if let Some(chain_commit) = self.chain.commit_for_block(commit.target_number) {
					assert_eq!(commit, chain_commit);
					return Default::default();
				}
				println!("{}: received {}", self.id, commit);

				if !self.chain.knows_about_block(commit.target_number) {
					// Requeue request for later, when we hopefully know about the block
					self.actions
						.push((current_tick + 10, Action::RequeueRequest(request.clone())));
					println!("{}: requesting block {}", self.id, commit.target_number);
					return vec![(request.0, Response::RequestBlock(commit.target_number))];
				}

				for (_block_number, previous_commit) in self.chain.commits() {
					if !self
						.chain
						.is_descendent(commit.target_number, previous_commit.target_number)
					{
						println!(
							"{}: received commit is not descendent of last finalized, \
							should trigger accountable safety protocol!",
							self.id
						);
					}
				}
			}
			Request::SendBlocks(blocks) => {
				println!("{}: received blocks", self.id);
				for block in blocks {
					if let Some(chain_block) = self.chain.get_block(block.number) {
						assert_eq!(&block, chain_block);
					} else {
						println!("{}: adding block {}", self.id, block);
						self.chain.add_block(block);
					}
				}
			}
		}
		Default::default()
	}

	pub fn handle_response(&mut self, response: (VoterId, Response), current_tick: usize) {
		match response.1 {
			Response::RequestBlock(block_number) => {
				self.actions.push((
					current_tick + 10,
					Action::SendBlock(response.0, block_number),
				));
			}
		}
	}
}

impl Display for Voter {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}
