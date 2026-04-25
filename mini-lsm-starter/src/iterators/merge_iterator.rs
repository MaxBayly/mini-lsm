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

use std::cmp::{self};
use std::collections::BinaryHeap;

use anyhow::Result;

use crate::key::KeySlice;

use super::StorageIterator;

struct HeapWrapper<I: StorageIterator>(pub usize, pub Box<I>);

impl<I: StorageIterator> PartialEq for HeapWrapper<I> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl<I: StorageIterator> Eq for HeapWrapper<I> {}

impl<I: StorageIterator> PartialOrd for HeapWrapper<I> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: StorageIterator> Ord for HeapWrapper<I> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.1
            .key()
            .cmp(&other.1.key())
            .then(self.0.cmp(&other.0))
            .reverse()
    }
}

/// Merge multiple iterators of the same type. If the same key occurs multiple times in some
/// iterators, prefer the one with smaller index.
pub struct MergeIterator<I: StorageIterator> {
    iters: BinaryHeap<HeapWrapper<I>>,
    current: Option<HeapWrapper<I>>,
}

impl<I: StorageIterator> MergeIterator<I> {
    pub fn create(iters: Vec<Box<I>>) -> Self {
        let mut idx = 0usize;
        let wrappers: Vec<HeapWrapper<I>> = iters
            .into_iter()
            .map(|inner_box| -> HeapWrapper<I> {
                let wrapper = HeapWrapper(idx, inner_box);
                idx += 1;
                wrapper
            })
            .filter(|it| it.1.is_valid())
            .collect();
        let mut inner_iters = BinaryHeap::from(wrappers);
        let smallest = inner_iters.pop();

        MergeIterator {
            iters: inner_iters,
            current: smallest,
        }
    }
}

impl<I: 'static + for<'a> StorageIterator<KeyType<'a> = KeySlice<'a>>> StorageIterator
    for MergeIterator<I>
{
    type KeyType<'a> = KeySlice<'a>;

    fn key(&self) -> KeySlice {
        self.current.as_ref().unwrap().1.key()
    }

    fn value(&self) -> &[u8] {
        self.current.as_ref().unwrap().1.value()
    }

    fn is_valid(&self) -> bool {
        self.current.is_some() && self.current.as_ref().unwrap().1.is_valid()
    }

    fn next(&mut self) -> Result<()> {
        match self.current.as_ref() {
            None => Ok(()),
            Some(wrapper) => {
                let old_key = wrapper.1.key();
                let iters = &mut self.iters;

                while let Some(mut head_iter) = iters.pop() {
                    // if the key of the next smallest iterator is not equal to the current key, then we
                    // move moved past this element
                    if head_iter.1.key() != old_key {
                        iters.push(head_iter);
                        break;
                    }

                    // if it is equal, advance that iterator
                    head_iter.1.next()?;
                    if head_iter.1.is_valid() {
                        iters.push(head_iter);
                    }
                }

                // take current from memory
                let mut curr = self.current.take();
                // advance current iterator
                curr.as_mut().unwrap().1.next()?;
                // place back in the heap if still valid
                match curr {
                    None => {}
                    Some(iterator) => {
                        if iterator.1.is_valid() {
                            iters.push(iterator)
                        }
                    }
                }

                // find the next smallest iterator
                match iters.pop() {
                    // if none, then iterator is finished
                    None => {
                        self.current = None;
                        return Ok(());
                    }
                    // if some, that is the new current (and is no longer in the heap after the pop())
                    Some(iter) => {
                        if iter.1.is_valid() {
                            self.current = Some(iter)
                        };
                    }
                };

                // store a ref to the key of the new current iterator
                let key = self.current.as_ref().unwrap().1.key();
                // infinitely peek_mut
                while let Some(mut head_iter) = iters.pop() {
                    // if the key of the next smallest iterator is not equal to the current key, then we
                    // move moved past this element
                    let idx = head_iter.0;
                    // println!("{:?}", idx);
                    if head_iter.1.key() != key {
                        if head_iter.1.is_valid() {
                            iters.push(head_iter)
                        }
                        break;
                    }

                    // if it is equal, advance that iterator
                    head_iter.1.next();
                    if head_iter.1.is_valid() {
                        iters.push(head_iter)
                    }
                }

                Ok(())
            }
        }
    }
}
