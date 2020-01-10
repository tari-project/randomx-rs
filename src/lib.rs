// Copyright 2019. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//! # randomx-rs
//!
//! The `randomx-rs` crate provides bindings to the `RandomX` proof-of-work (PoW) system as well
//! as the functionality to utilize these bindings
//!
mod bindings;
#[macro_use]
extern crate bitflags;
extern crate libc;

use bindings::{
    randomx_alloc_cache, randomx_alloc_dataset, randomx_cache, randomx_calculate_hash,
    randomx_create_vm, randomx_dataset, randomx_dataset_item_count, randomx_destroy_vm,
    randomx_get_dataset_memory, randomx_init_cache, randomx_init_dataset, randomx_release_cache,
    randomx_release_dataset, randomx_vm, randomx_vm_set_cache, randomx_vm_set_dataset,
    RANDOMX_DATASET_ITEM_SIZE, RANDOMX_HASH_SIZE,
};

use derive_error::Error;
use libc::{c_char, c_ulong, c_void};
use std::mem;
use std::ptr;

bitflags! {
/// Indicates to the RandomX library which configuration options to use
    pub struct RandomXFlag: u32 {
    /// All flags not set, works on all platforms, however is the slowest
        const FLAG_DEFAULT =     0b00000000;
    /// Allocate memory in large pages
        const FLAG_LARGE_PAGES = 0b00000001;
    /// Use hardware accelerated AES
        const FLAG_HARD_AES =    0b00000010;
    /// Use the full dataset
        const FLAG_FULL_MEM =    0b00000100;
    /// Use JIT compilation support
        const FLAG_JIT =         0b00001000;
    /// When combined with FLAG_JIT, the JIT pages are never writable and executable at the
    /// same time
        const FLAG_SECURE =      0b00010000;
    /// Optimize Argon2 for CPUs with the SSSE3 instruction set
        const FLAG_ARGON2_SSSE3 =0b00100000;
    /// Optimize Argon2 for CPUs with the AVX2 instruction set
        const FLAG_ARGON2_AVX2  =0b01000000;
    }
}

#[derive(Debug, Clone, Error)]
/// Custom error enum
pub enum RandomXError {
    /// Problem creating the RandomX object
    CreationError,
    /// Problem with configuration flags
    FlagConfigError,
    /// Problem running RandomX
    Other,
}

#[derive(Debug)]
/// Cache structure
pub struct RandomXCache {
    cache: *mut randomx_cache,
}

impl Drop for RandomXCache {
    /// De-allocates memory for the `cache` object
    fn drop(&mut self) {
        unsafe {
            randomx_release_cache(self.cache);
        }
    }
}

impl RandomXCache {
    /// Creates a new cache object, allocates memory to the `cache` object and initializes it with
    /// he key value, error on failure
    ///
    /// `flags` is any combination of the following two flags:
    /// * FLAG_LARGE_PAGES
    /// * FLAG_JIT
    ///
    /// and (optionally) one of the following flags (depending on instruction set supported)
    /// * FLAG_ARGON2_SSSE3
    /// * FLAG_ARGON2_AVX2
    ///
    /// `key` is a sequence of characters used to initialize SuperScalarHash
    pub fn new(flags: RandomXFlag, key: &str) -> Result<RandomXCache, RandomXError> {
        if key.len() == 0 {
            return Err(RandomXError::CreationError);
        };
        let test = unsafe { randomx_alloc_cache(flags.bits) };
        if test.is_null() {
            Err(RandomXError::CreationError)
        } else {
            let result = RandomXCache { cache: test };
            let key_ptr = key.as_bytes().as_ptr() as *mut c_void;
            let key_size = key.as_bytes().len() * mem::size_of::<*const c_char>();
            unsafe {
                //no way to check if this fails, c code does not return anything
                randomx_init_cache(result.cache, key_ptr, key_size);
            }
            Ok(result)
        }
    }
}

#[derive(Debug)]
/// Dataset structure
pub struct RandomXDataset {
    dataset: *mut randomx_dataset,
    dataset_start: c_ulong,
    dataset_count: c_ulong,
}

impl Drop for RandomXDataset {
    /// De-allocates memory for the `dataset` object
    fn drop(&mut self) {
        unsafe {
            randomx_release_dataset(self.dataset);
        }
    }
}

impl RandomXDataset {
    /// Creates a new dataset object, allocates memory to the `dataset` object and initializes it,
    /// error on failure
    ///
    /// `flags` is one of the following:
    /// * FLAG_DEFAULT
    /// * FLAG_LARGE_PAGES
    /// `cache` is a cache object
    /// `start` is the item number where initialization should start, recommended to pass in 0
    pub fn new(
        flags: RandomXFlag,
        cache: &RandomXCache,
        start: c_ulong,
    ) -> Result<RandomXDataset, RandomXError> {
        let count = c_ulong::from(RANDOMX_DATASET_ITEM_SIZE - 1) - start;
        let test = unsafe { randomx_alloc_dataset(flags.bits) };
        if test.is_null() {
            Err(RandomXError::CreationError)
        } else {
            let result = RandomXDataset {
                dataset: test,
                dataset_start: start,
                dataset_count: count,
            };
            let item_count = match result.count() {
                Ok(v) => v,
                Err(_) => return Err(RandomXError::CreationError),
            };
            // Mirror the assert checks inside randomx_init_dataset call
            if !((start < (item_count as c_ulong) && count <= (item_count as c_ulong))
                || (start + (item_count as c_ulong) <= count))
            {
                return Err(RandomXError::CreationError);
            }
            unsafe {
                //no way to check if this fails, c code does not return anything
                randomx_init_dataset(
                    result.dataset,
                    cache.cache,
                    start as c_ulong,
                    count as c_ulong,
                );
            }
            Ok(result)
        }
    }

    /// Returns the number of items in the `dataset` or an error on failure
    pub fn count(&self) -> Result<u64, RandomXError> {
        match unsafe { randomx_dataset_item_count() } {
            0 => Err(RandomXError::Other),
            x => Ok(x as u64),
        }
    }

    /// Returns the values of the internal memory buffer of the `dataset` or an error on failure
    pub fn get_data(&self) -> Result<Vec<u8>, RandomXError> {
        let memory = unsafe { randomx_get_dataset_memory(self.dataset) };
        if memory.is_null() {
            return Err(RandomXError::Other);
        }
        let mut result: Vec<u8> = vec![0u8; self.dataset_count as usize];
        unsafe {
            libc::memcpy(
                result.as_mut_ptr() as *mut c_void,
                memory,
                self.dataset_count as usize,
            );
        }
        Ok(result)
    }
}

#[derive(Debug)]
/// VM structure
pub struct RandomXVM {
    vm: *mut randomx_vm,
}

impl Drop for RandomXVM {
    /// De-allocates memory for the `VM` object
    fn drop(&mut self) {
        unsafe {
            randomx_destroy_vm(self.vm);
        }
    }
}

impl RandomXVM {
    /// Creates a new `VM` and initializes it, error on failure
    ///
    /// `flags` is any combination of the following 5 flags:
    /// * FLAG_LARGE_PAGES
    /// * FLAG_HARD_AES
    /// * FLAG_FULL_MEM
    /// * FLAG_JIT
    /// * FLAG_SECURE
    ///
    /// Or
    /// * FLAG_DEFAULT
    ///
    /// `cache` is a cache object, optional if FLAG_FULL_MEM is used
    /// `dataset` is a dataset object, optional if FLAG_FULL_MEM is not used
    pub fn new(
        flags: RandomXFlag,
        cache: &RandomXCache, //TODO Update to optional, check flags, on error return FlagConfigError
        dataset: Option<&RandomXDataset>, //TODO check flags, on error return FlagConfigError
    ) -> Result<RandomXVM, RandomXError> {
        let test: *mut randomx_vm;
        match dataset {
            Some(data) => unsafe {
                test = randomx_create_vm(flags.bits, cache.cache, data.dataset)
            },
            None => unsafe { test = randomx_create_vm(flags.bits, cache.cache, ptr::null_mut()) },
        }
        if test.is_null() {
            return Err(RandomXError::CreationError);
        }
        let result = RandomXVM { vm: test };
        Ok(result)
    }

    /// Re-initializes the `VM` with a new cache
    pub fn reinit_cache(&self, cache: &RandomXCache) {
        //no way to check if this fails, c code does not return anything
        unsafe {
            randomx_vm_set_cache(self.vm, cache.cache);
        }
    }

    /// Re-initializes the `VM` with a new dataset
    pub fn reinit_dataset(&self, dataset: &RandomXDataset) {
        //no way to check if this fails, c code does not return anything
        unsafe {
            randomx_vm_set_dataset(self.vm, dataset.dataset);
        }
    }

    /// Calculates a RandomX hash value and returns it, error on failure
    ///
    /// `input` is a sequence of characters to be hashed
    pub fn calculate_hash(&self, input: &str) -> Result<Vec<u8>, RandomXError> {
        if input.len() == 0 {
            return Err(RandomXError::Other);
        };
        let size_input = input.as_bytes().len() * mem::size_of::<*const c_char>();
        let input_ptr = input.as_bytes().as_ptr() as *mut c_void;
        let arr = [0; RANDOMX_HASH_SIZE as usize];
        let output_ptr = arr.as_ptr() as *mut c_void;
        unsafe {
            randomx_calculate_hash(self.vm, input_ptr, size_input, output_ptr);
        }
        // if this failed, arr should still be empty
        if arr == [0; RANDOMX_HASH_SIZE as usize] {
            return Err(RandomXError::Other);
        }
        let result = arr.to_vec();
        Ok(result)
    }

    //TODO randomx_get_flags // get recommended flags for machine

    //TODO paired functions to calculate multiple RandomX hashes more efficiently
    //TODO pub fn randomx_calculate_hash_first // called for first input value
    //TODO pub fn randomx_calculate_hash_next // outputs hash of previous input
}

#[cfg(test)]
mod tests {
    use crate::{RandomXCache, RandomXDataset, RandomXFlag, RandomXVM};

    #[test]
    fn lib_alloc_cache() {
        let flags = RandomXFlag::FLAG_DEFAULT;
        let key = "Key";
        let cache = RandomXCache::new(flags, key);
        if let Err(i) = cache {
            panic!(format!("Failed to allocate cache, {}", i));
        }
        drop(cache);
    }

    #[test]
    fn lib_alloc_dataset() {
        let flags = RandomXFlag::FLAG_DEFAULT;
        let key = "Key";
        let cache = RandomXCache::new(flags, key).unwrap();
        let dataset = RandomXDataset::new(flags, &cache, 0);
        if let Err(i) = dataset {
            panic!(format!("Failed to allocate dataset, {}", i));
        }
        drop(dataset);
        drop(cache);
    }

    #[test]
    fn lib_alloc_vm() {
        let flags = RandomXFlag::FLAG_DEFAULT;
        let key = "Key";
        let cache = RandomXCache::new(flags, key).unwrap();
        let mut vm = RandomXVM::new(flags, &cache, None);
        if let Err(i) = vm {
            panic!(format!("Failed to allocate vm, {}", i));
        }
        drop(vm);
        let dataset = RandomXDataset::new(flags, &cache, 0).unwrap();
        vm = RandomXVM::new(flags, &cache, Some(&dataset));
        if let Err(i) = vm {
            panic!(format!("Failed to allocate vm, {}", i));
        }
        drop(dataset);
        drop(cache);
        drop(vm);
    }

    #[test]
    fn lib_dataset_memory() {
        let flags = RandomXFlag::FLAG_DEFAULT;
        let key = "Key";
        let cache = RandomXCache::new(flags, key).unwrap();
        let dataset = RandomXDataset::new(flags, &cache, 0).unwrap();
        let memory = dataset.get_data().unwrap_or(std::vec::Vec::new());
        if memory.len() == 0 {
            panic!("Failed to get dataset memory");
        }
        let vec = vec![0u8; memory.len() as usize];
        assert_ne!(memory, vec);
        drop(dataset);
        drop(cache);
    }

    #[test]
    fn lib_calculate_hash() {
        let flags = RandomXFlag::FLAG_DEFAULT;
        let key = "Key";
        let input = "Input";
        let cache = RandomXCache::new(flags, key).unwrap();
        let vm = RandomXVM::new(flags, &cache, None).unwrap();
        let hash = vm.calculate_hash(input).expect("no data");
        let vec = vec![0u8; hash.len() as usize];
        assert_ne!(hash, vec);
        vm.reinit_cache(&cache);
        let dataset = RandomXDataset::new(flags, &cache, 0).unwrap();
        vm.reinit_dataset(&dataset);
        let hash = vm.calculate_hash(input).expect("no data");
        assert_ne!(hash, vec);
        drop(dataset);
        drop(cache);
        drop(vm);
    }
}
