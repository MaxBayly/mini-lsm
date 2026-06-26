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

use std::sync::Arc;

use anyhow::{Result, anyhow};

use super::{BlockMeta, SsTable, SsTableBuilder};
use crate::{block::BlockIterator, iterators::StorageIterator, key::KeySlice};

/// An iterator over the contents of an SSTable.
pub struct SsTableIterator {
    table: Arc<SsTable>,
    blk_iter: BlockIterator,
    blk_idx: usize,
}

impl SsTableIterator {
    /// Create a new iterator and seek to the first key-value pair in the first data block.
    pub fn create_and_seek_to_first(table: Arc<SsTable>) -> Result<Self> {
        let first_block = table.read_block(0)?;
        let block_iterator = BlockIterator::create_and_seek_to_first(first_block);
        let mut ssti = SsTableIterator {
            table: table,
            blk_iter: block_iterator,
            blk_idx: 0,
        };
        ssti.seek_to_first()?;
        Ok(ssti)
    }

    /// Seek to the first key-value pair in the first data block.
    pub fn seek_to_first(&mut self) -> Result<()> {
        let first_block = self.table.read_block(0)?;
        let block_iterator = BlockIterator::create_and_seek_to_first(first_block);
        self.blk_iter = block_iterator;
        self.blk_idx = 0;
        Ok(())
    }

    /// Create a new iterator and seek to the first key-value pair which >= `key`.
    pub fn create_and_seek_to_key(table: Arc<SsTable>, key: KeySlice) -> Result<Self> {
        let first_block = table.read_block(0)?;
        let block_iterator = BlockIterator::create_and_seek_to_first(first_block);
        let mut ssti = SsTableIterator {
            table: table,
            blk_iter: block_iterator,
            blk_idx: 0,
        };
        ssti.seek_to_key(key)?;
        Ok(ssti)
    }

    /// Seek to the first key-value pair which >= `key`.
    /// Note: You probably want to review the handout for detailed explanation when implementing
    /// this function.
    pub fn seek_to_key(&mut self, key: KeySlice) -> Result<()> {
        let mut target_block_idx = self.table.find_block_idx(key);
        let mut block = self.table.read_block_cached(target_block_idx)?;
        let mut block_iterator = BlockIterator::create_and_seek_to_key(block, key);
        if !block_iterator.is_valid() && target_block_idx + 1 < self.table.num_of_blocks() {
            target_block_idx += 1;
            block = self.table.read_block(target_block_idx)?;
            block_iterator = BlockIterator::create_and_seek_to_key(block, key);
        }
        self.blk_iter = block_iterator;
        self.blk_idx = target_block_idx;

        Ok(())
    }
}

impl StorageIterator for SsTableIterator {
    type KeyType<'a> = KeySlice<'a>;

    /// Return the `key` that's held by the underlying block iterator.
    fn key(&self) -> KeySlice {
        self.blk_iter.key()
    }

    /// Return the `value` that's held by the underlying block iterator.
    fn value(&self) -> &[u8] {
        self.blk_iter.value()
    }

    /// Return whether the current block iterator is valid or not.
    fn is_valid(&self) -> bool {
        self.blk_iter.is_valid()
    }

    /// Move to the next `key` in the block.
    /// Note: You may want to check if the current block iterator is valid after the move.
    fn next(&mut self) -> Result<()> {
        self.blk_iter.next();
        if (self.blk_iter.is_valid()) {
            Ok(())
        } else {
            let next_idx = self.blk_idx + 1;
            if next_idx >= self.table.num_of_blocks() {
                return Ok(());
            }
            let block = self.table.read_block_cached(next_idx)?;
            self.blk_idx = next_idx;
            Ok(self.blk_iter = BlockIterator::create_and_seek_to_first(block))
        }
    }
}
