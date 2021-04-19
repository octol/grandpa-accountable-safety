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
	block::{BlockNumber, Block},
	voting::{Commit, Precommit, RoundNumber},
	voter::VoterId
};

#[derive(Debug, Clone)]
pub enum Request {
	HereIsCommit(RoundNumber, Commit),
	HereAreBlocks(Vec<Block>),
	WhyDidEstimateForRoundNotIncludeBlock(RoundNumber, BlockNumber),
}

#[derive(Debug, Clone)]
pub enum Response {
	RequestBlock(BlockNumber),
	PrecommitsForEstimate(RoundNumber, Vec<Precommit>),
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

