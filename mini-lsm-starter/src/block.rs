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

mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::{BufMut, Bytes};
pub use iterator::BlockIterator;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the course
    /// Note: You may want to recheck if any of the expected field is missing from your output
    pub fn encode(&self) -> Bytes {
        let mut buf = self.data.clone();
        for &offset in &self.offsets {
            buf.put_u16(offset);
        }
        buf.put_u16(self.offsets.len() as u16);
        Bytes::copy_from_slice(&buf)
    }

    /// Decode from the data layout, transform the input `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        let entry_count_bytes: &[u8] = &data[data.len() - 2..];
        let entry_count = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]);
        let offset_bytes = &data[data.len() - (entry_count * 2) as usize - 2..data.len() - 2];
        let mut offsets = vec![];
        for chunk in offset_bytes.chunks_exact(2) {
            let offset = u16::from_be_bytes([chunk[0], chunk[1]]);
            offsets.push(offset);
        }

        let data_bytes = &data[0..(data.len() - (entry_count * 2) as usize - 2)];

        Block {
            data: data_bytes.into(),
            offsets,
        }
    }
}
