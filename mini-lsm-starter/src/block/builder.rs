// Copyright (c) 2022-2025 Alex Chi Z
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

#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use crate::key::{KeySlice, KeyVec};
use bytes::BufMut;

use super::Block;

/// Builds a block.
pub struct BlockBuilder {
    /// Offsets of each key-value entries.
    offsets: Vec<u16>,
    /// All serialized key-value pairs in the block.
    data: Vec<u8>,
    /// The expected block size.
    block_size: usize,
    /// The first key in the block
    first_key: KeyVec,
}

impl BlockBuilder {
    /// Creates a new block builder.
    pub fn new(block_size: usize) -> Self {
        BlockBuilder {
            offsets: vec![],
            data: vec![],
            block_size,
            first_key: KeyVec::new(),
        }
    }

    /// Adds a key-value pair to the block. Returns false when the block is full.
    /// You may find the `bytes::BufMut` trait useful for manipulating binary data.
    #[must_use]
    pub fn add(&mut self, key: KeySlice, value: &[u8]) -> bool {
        let length = self.data.len() + (self.offsets.len() * 2);
        // old length + 2 (key len bytes) + key.len (key bytes) + 2 (value len bytes) + value.len (value bytes) + 2 (one more offset u16) +  2 (entry count)
        let new_length = length + 2 + key.len() + 2 + value.len() + 2 + 2;
        if new_length > self.block_size && !self.offsets.is_empty() {
            return false;
        }

        let key_length = key.into_inner();
        let val_length = value.len();
        let mut buf = vec![];
        buf.put_u16(key.len() as u16);
        buf.put(key.into_inner());
        buf.put_u16(val_length as u16);
        buf.put(value);
        let offset = u16::try_from(self.data.len()).expect("offset too large");
        self.offsets.push(offset);

        self.data.append(&mut buf);
        true
    }

    /// Check if there is no key-value pair in the block.
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    /// Finalize the block.
    pub fn build(self) -> Block {
        // let mut offsets: Vec<u16> = vec![];
        // let mut offset: u16 = 0;
        // for entry in &self.data {
        //     offsets.push(offset);
        //     offset += entry.to_usize() as u16;
        // }

        Block {
            data: self.data,
            offsets: self.offsets,
        }
    }
}
