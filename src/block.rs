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

use std::fmt::{Display, Formatter};

pub type BlockNumber = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
	pub number: BlockNumber,
	pub parent: BlockNumber,
}

impl Block {
	pub fn new(number: BlockNumber, parent: BlockNumber) -> Self {
		Self { number, parent }
	}

	pub fn is_genesis(&self) -> bool {
		self.number == 0 && self.parent == 0
	}
}

impl Display for Block {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "Block({}, parent: {})", self.number, self.parent)
	}
}
