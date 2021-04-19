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

//! Accountable Safety for GRANDPA
//! ==============================
//!
//! Accountable Safety for GRANDPA is a synchronous interactive protocol for tracking down and
//! proving after the fact when participants misbehave. The idea is that even is more than 1/3 of
//! participants misbehave and finalize conflicting forks, they will not get away with and will get
//! their stake slashed.
//!
//! In the [GRANDPA paper](https://arxiv.org/pdf/2007.01560.pdf) there is a proof by construction for
//! showing that if two blocks B and B' for which valid commit messages were sent, but do not lie on
//! the same chain, then there are at least f + 1 Byzantine voters. The proof itself then provides
//! the procedure for tracking down this set of misbehaving voters.
//!
//! Definitions
//! ===========
//!
//! We refer to the GRANDPA paper for in-depth material, it is still useful to restate a some of
//! the more important definitions here.
//!
//! GHOST Function
//! --------------
//! The function g(S) takes the set of votes and returns the block B with the highest block number
//! such that S has a supermajority for B.
//!
//! Estimate
//! --------
//! E_{r,v} is voter v's estimate of what might have been finalized in round r, given by the last
//! block in the chain with head g(V_{r,v}) for which it is possible for C_{r,v} to have a
//! supermajority.
//!
//! Completable
//! -----------
//! If either E_{r,v} < g(V_{r,v}) or it is impossible for C_{r,v} to have a supermajority for any
//! children of g(V_{r,v}), then we say that v sees that round r as completable.
//!
//! In other words, when E_{r,v} contains everything that could have been finalized in round r.
//!
//! E_{r,v} having supermajority means that E_{r,v} < g(V_{r,v}).
//! WIP(JON): how?
//!
//! Outline of the Procedure
//! ========================
//!
//! Step 0.
//! -------
//!
//! The first step is detecting blocks B and B' on two different branches being finalized.
//! We assume B' was finalized in a later round r' than B, which was finalized in round r.
//! That is, r'> r.
//!
//! ```text
//! o-o-o-B
//!  \o-o-B'
//! ```
//!
//! Step 1. start asking questions about B'
//! ---------------------------------------
//!
//! Q: Why the estimate did not include B when prevoting for B'
//! A: A set S of prevotes or a set S of precommits of the preceding round.
//!    In either case such that it is impossible for S to have a supermajority for B.
//!
//! (Repeat for each round back to round r+1.)
//!
//! Step 2. reach the round after which B was finalized
//! ---------------------------------------------------
//!
//! The reply for round r+1 will contain a set S of either prevotes or precommits
//! - If precommits: take union with precommits in commit msg for B to find equivocators.
//! - If prevotes: ask the precommitters for B.
//!
//! Step 3. instead ask the precommitters for B
//! -------------------------------------------
//!
//! Q: Ask all precommitters in the in commit msg for B, which prevotes have you seen?
//! A: A set T of prevotes with a supermajority for B.
//!    Take the union with S and find the equivocators.
//!

mod action;
mod block;
mod chain;
mod message;
mod protocol;
mod voter;
mod voting;
pub mod world;

#[cfg(test)]
mod tests;
