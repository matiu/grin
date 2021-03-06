// Copyright 2016 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Base types that the block chain pipeline requires.

use core::core::hash::Hash;
use core::core::{Block, BlockHeader};
use core::ser;

/// The lineage of a fork, defined as a series of numbers. Each new branch gets
/// a new number that gets added to a fork's ancestry to form a new fork.
/// Example:
///   head [1] -> fork1 [1, 2]
///               fork2 [1, 3]
#[derive(Debug, Clone)]
pub struct Lineage(Vec<u32>);

impl Lineage {
	/// New lineage initialized just with branch 0
	pub fn new() -> Lineage {
		Lineage(vec![0])
	}
	/// The last branch that was added to the lineage. Also the only branch
	/// that's
	/// unique to this lineage.
	pub fn last_branch(&self) -> u32 {
		*self.0.last().unwrap()
	}
}

/// Serialization for lineage, necessary to serialize fork tips.
impl ser::Writeable for Lineage {
	fn write(&self, writer: &mut ser::Writer) -> Result<(), ser::Error> {
		try!(writer.write_u32(self.0.len() as u32));
		for num in &self.0 {
			try!(writer.write_u32(*num));
		}
		Ok(())
	}
}
/// Deserialization for lineage, necessary to deserialize fork tips.
impl ser::Readable<Lineage> for Lineage {
	fn read(reader: &mut ser::Reader) -> Result<Lineage, ser::Error> {
		let len = try!(reader.read_u32());
		let mut branches = Vec::with_capacity(len as usize);
		for _ in 0..len {
			branches.push(try!(reader.read_u32()));
		}
		Ok(Lineage(branches))
	}
}

/// The tip of a fork. A handle to the fork ancestry from its leaf in the
/// blockchain tree. References both the lineage of the fork as well as its max
/// height and its latest and previous blocks for convenience.
#[derive(Debug, Clone)]
pub struct Tip {
	/// Height of the tip (max height of the fork)
	pub height: u64,
	/// Last block pushed to the fork
	pub last_block_h: Hash,
	/// Block previous to last
	pub prev_block_h: Hash,
	/// Lineage in branch numbers of the fork
	pub lineage: Lineage,
}

impl Tip {
	/// Creates a new tip at height zero and the provided genesis hash.
	pub fn new(gbh: Hash) -> Tip {
		Tip {
			height: 0,
			last_block_h: gbh,
			prev_block_h: gbh,
			lineage: Lineage::new(),
		}
	}

	/// Append a new block hash to this tip, returning a new updated tip.
	pub fn append(&self, bh: Hash) -> Tip {
		Tip {
			height: self.height + 1,
			last_block_h: bh,
			prev_block_h: self.last_block_h,
			lineage: self.lineage.clone(),
		}
	}
}

/// Serialization of a tip, required to save to datastore.
impl ser::Writeable for Tip {
	fn write(&self, writer: &mut ser::Writer) -> Result<(), ser::Error> {
		try!(writer.write_u64(self.height));
		try!(writer.write_fixed_bytes(&self.last_block_h));
		try!(writer.write_fixed_bytes(&self.prev_block_h));
		self.lineage.write(writer)
	}
}

impl ser::Readable<Tip> for Tip {
	fn read(reader: &mut ser::Reader) -> Result<Tip, ser::Error> {
		let height = try!(reader.read_u64());
		let last = try!(Hash::read(reader));
		let prev = try!(Hash::read(reader));
		let line = try!(Lineage::read(reader));
		Ok(Tip {
			height: height,
			last_block_h: last,
			prev_block_h: prev,
			lineage: line,
		})
	}
}

#[derive(Debug)]
pub enum Error {
	/// Couldn't find what we were looking for
	NotFoundErr,
	/// Error generated by the underlying storage layer
	StorageErr(String),
}

/// Trait the chain pipeline requires an implementor for in order to process
/// blocks.
pub trait ChainStore: Send + Sync {
	/// Get the tip that's also the head of the chain
	fn head(&self) -> Result<Tip, Error>;

	/// Block header for the chain head
	fn head_header(&self) -> Result<BlockHeader, Error>;

	/// Gets a block header by hash
	fn get_block_header(&self, h: &Hash) -> Result<BlockHeader, Error>;

	/// Save the provided block in store
	fn save_block(&self, b: &Block) -> Result<(), Error>;

	/// Save the provided tip as the current head of our chain
	fn save_head(&self, t: &Tip) -> Result<(), Error>;

	/// Save the provided tip without setting it as head
	fn save_tip(&self, t: &Tip) -> Result<(), Error>;
}

/// Bridge between the chain pipeline and the rest of the system. Handles
/// downstream processing of valid blocks by the rest of the system, most
/// importantly the broadcasting of blocks to our peers.
pub trait ChainAdapter {
	/// The blockchain pipeline has accepted this block as valid and added
	/// it to our chain.
	fn block_accepted(&self, b: &Block);
}

pub struct NoopAdapter { }
impl ChainAdapter for NoopAdapter {
	fn block_accepted(&self, b: &Block) {}
}
