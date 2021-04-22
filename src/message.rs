// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::{
	block::{Block, BlockNumber},
	protocol::QueryResponse,
	voter::VoterId,
	voting::{Commit, RoundNumber},
};

#[derive(Debug, Clone)]
pub enum Request {
	HereIsCommit(RoundNumber, Commit),
	HereAreBlocks(Vec<Block>),
	WhyDidEstimateForRoundNotIncludeBlock(RoundNumber, BlockNumber),
	WhichPrevotesSeenInRound(RoundNumber),
}

#[derive(Debug, Clone)]
pub enum Response {
	RequestBlock(BlockNumber),
	ExplainEstimate(RoundNumber, QueryResponse),
	PrevotesSeen(RoundNumber, QueryResponse),
}

#[derive(Debug, Clone)]
pub enum Payload {
	Request(Request),
	Response(Response),
}

impl Payload {
	pub fn request(&self) -> Option<&Request> {
		match self {
			Payload::Request(request) => Some(request),
			Payload::Response(..) => None,
		}
	}

	pub fn response(&self) -> Option<&Response> {
		match self {
			Payload::Request(..) => None,
			Payload::Response(response) => Some(response),
		}
	}
}

#[derive(Debug)]
pub struct Message {
	pub sender: VoterId,
	pub receiver: VoterId,
	pub content: Payload,
}
