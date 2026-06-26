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

use std::mem;
use std::path::Path;
use std::sync::Arc;

use super::{BlockMeta, FileObject, SsTable};
use crate::key::KeyBytes;
use crate::{block::BlockBuilder, key::KeySlice, lsm_storage::BlockCache};
use anyhow::{Result, anyhow};
use bytes::{Buf, BufMut, Bytes};

/// Builds an SSTable from key-value pairs.
pub struct SsTableBuilder {
    builder: BlockBuilder,
    first_key: Vec<u8>,
    last_key: Vec<u8>,
    data: Vec<u8>,
    pub(crate) meta: Vec<BlockMeta>,
    block_size: usize,
}

impl SsTableBuilder {
    /// Create a builder based on target block size.
    pub fn new(block_size: usize) -> Self {
        SsTableBuilder {
            builder: BlockBuilder::new(block_size),
            first_key: Vec::new(),
            last_key: Vec::new(),
            data: Vec::new(),
            meta: Vec::new(),
            block_size,
        }
    }

    /// Adds a key-value pair to SSTable.
    ///
    /// Note: You should split a new block when the current block is full.(`std::mem::replace` may
    /// be helpful here)
    pub fn add(&mut self, key: KeySlice, value: &[u8]) {
        if self.first_key.is_empty() {
            self.first_key = Vec::from(key.into_inner())
        }
        if self.builder.add(key, value) {
            self.last_key = Vec::from(key.into_inner());
        } else {
            self.build_and_flush();
            self.first_key = Vec::from(key.into_inner());
            self.last_key = Vec::from(key.into_inner());
            debug_assert!(self.builder.add(key, value));
        }
    }

    fn build_and_flush(&mut self) {
        let builder = mem::replace(&mut self.builder, BlockBuilder::new(self.block_size));
        let block = builder.build();
        let meta = BlockMeta {
            offset: self.data.len(),
            first_key: KeyBytes::from_bytes(Bytes::from(self.first_key.clone())),
            last_key: KeyBytes::from_bytes(Bytes::from(self.last_key.clone())),
        };
        self.data.extend_from_slice(block.encode().as_ref());
        self.meta.push(meta);
        self.first_key = Vec::new();
        self.last_key = Vec::new();
    }

    /// Get the estimated size of the SSTable.
    ///
    /// Since the data blocks contain much more data than meta blocks, just return the size of data
    /// blocks here.
    pub fn estimated_size(&self) -> usize {
        unimplemented!()
    }

    /// Builds the SSTable and writes it to the given path. Use the `FileObject` structure to manipulate the disk objects.
    pub fn build(
        #[allow(unused_mut)] mut self,
        id: usize,
        block_cache: Option<Arc<BlockCache>>,
        path: impl AsRef<Path>,
    ) -> Result<SsTable> {
        if !self.builder.is_empty() {
            self.build_and_flush();
        }
        if self.meta.is_empty() {
            return Err(anyhow!("cannot build an SST from empty input"));
        }

        let len = self.data.len();
        let first_block_key = Vec::from(self.meta[0].first_key.raw_ref());
        let last_key = Vec::from(self.meta.last().unwrap().last_key.raw_ref());

        let mut meta_buffer: Vec<u8> = Vec::new();
        BlockMeta::encode_block_meta(&*self.meta, &mut meta_buffer);
        self.data.append(&mut meta_buffer);
        self.data.put_u32(len as u32); // meta block offset

        let fo = FileObject::create(path.as_ref(), self.data)?;
        Ok(SsTable {
            file: fo,
            block_meta: self.meta,
            block_meta_offset: len,
            id,
            block_cache,
            first_key: KeyBytes::from_bytes(Bytes::from(first_block_key)),
            last_key: KeyBytes::from_bytes(Bytes::from(last_key)),
            bloom: None,
            max_ts: 0,
        })
    }

    #[cfg(test)]
    pub(crate) fn build_for_test(self, path: impl AsRef<Path>) -> Result<SsTable> {
        self.build(0, None, path)
    }
}
