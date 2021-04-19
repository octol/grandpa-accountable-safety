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
	action::Action,
	chain::Chain,
	message::{Message, Payload, Request},
	voter::{Voter, VoterId},
	voting::{Commit, VoterSet, VotingRound, VotingRounds},
};
use std::collections::BTreeMap;

const MAX_TICKS: usize = 5000;

pub struct World {
	voters: BTreeMap<VoterId, Voter>,
	current_tick: usize,
}

impl World {
	pub fn new(voters: BTreeMap<VoterId, Voter>) -> Self {
		Self {
			voters,
			current_tick: 0,
		}
	}

	pub fn list_commits(&self) {
		for (_, voter) in &self.voters {
			println!("{}:", voter);
			voter.list_commits();
		}
	}

	pub fn tick(&mut self) {
		self.current_tick += 1;
	}

	pub fn completed(&self) -> bool {
		self.current_tick >= MAX_TICKS
	}

	pub fn process_actions(&mut self) -> Vec<Message> {
		let mut requests = Vec::new();
		for (_, voter) in &mut self.voters {
			let voter_requests = voter.process_actions(self.current_tick);
			requests.extend(voter_requests);
		}
		requests
	}

	pub fn handle_requests(&mut self, requests: Vec<Message>) -> Vec<Message> {
		let mut responses = Vec::new();
		for Message {
			sender,
			receiver,
			content,
		} in requests
		{
			let request = content.request();
			let receiving_voter = self
				.voters
				.get_mut(&receiver)
				.expect("all requests are to known voters");
			let voter_responses = receiving_voter
				.handle_request((sender, request.clone()), self.current_tick)
				.into_iter()
				.map(|(response_receiver, res)| Message {
					receiver: response_receiver,
					sender: receiver.clone(),
					content: Payload::Response(res),
				});
			responses.extend(voter_responses);
		}
		responses
	}

	pub fn handle_responses(&mut self, responses: Vec<Message>) {
		for Message {
			sender,
			receiver,
			content,
		} in responses
		{
			let response = content.response();
			let receiving_voter = self
				.voters
				.get_mut(&receiver)
				.expect("all responses are to known voters");
			receiving_voter.handle_response((sender, response.clone()), self.current_tick);
		}
	}
}
