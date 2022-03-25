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
//! as the functionality to utilize these bindings.
mod bindings;
use std::{convert::TryFrom, num::TryFromIntError, ptr, sync::Arc};

use bindings::{
    randomx_alloc_cache,
    randomx_alloc_dataset,
    randomx_cache,
    randomx_calculate_hash,
    randomx_create_vm,
    randomx_dataset,
    randomx_dataset_item_count,
    randomx_destroy_vm,
    randomx_get_dataset_memory,
    randomx_init_cache,
    randomx_init_dataset,
    randomx_release_cache,
    randomx_release_dataset,
    randomx_vm,
    randomx_vm_set_cache,
    randomx_vm_set_dataset,
    RANDOMX_DATASET_ITEM_SIZE,
    RANDOMX_HASH_SIZE,
};
use bitflags::bitflags;
use libc::{c_ulong, c_void};
use thiserror::Error;

use crate::bindings::{
    randomx_calculate_hash_first,
    randomx_calculate_hash_last,
    randomx_calculate_hash_next,
    randomx_get_flags,
};

bitflags! {
/// Indicates to the RandomX library which configuration options to use.
    pub struct RandomXFlag: u32 {
    /// All flags not set, works on all platforms, however is the slowest
        const FLAG_DEFAULT      =0b0000_0000;
    /// Allocate memory in large pages
        const FLAG_LARGE_PAGES  =0b0000_0001;
    /// Use hardware accelerated AES
        const FLAG_HARD_AES     =0b0000_0010;
    /// Use the full dataset
        const FLAG_FULL_MEM     =0b0000_0100;
    /// Use JIT compilation support
        const FLAG_JIT          =0b0000_1000;
    /// When combined with FLAG_JIT, the JIT pages are never writable and executable at the
    /// same time
        const FLAG_SECURE       =0b0001_0000;
    /// Optimize Argon2 for CPUs with the SSSE3 instruction set
        const FLAG_ARGON2_SSSE3 =0b0010_0000;
    /// Optimize Argon2 for CPUs with the AVX2 instruction set
        const FLAG_ARGON2_AVX2  =0b0100_0000;
    /// Optimize Argon2 for CPUs without the AVX2 or SSSE3 instruction sets
        const FLAG_ARGON2       =0b0110_0000;
    }
}

impl RandomXFlag {
    /// Returns the recommended flags to be used.
    ///
    /// Does not include:
    /// * FLAG_LARGE_PAGES
    /// * FLAG_FULL_MEM
    /// * FLAG_SECURE
    ///
    /// The above flags need to be set manually, if required.
    pub fn get_recommended_flags() -> RandomXFlag {
        // c code will always return a value
        RandomXFlag {
            bits: unsafe { randomx_get_flags() },
        }
    }
}

impl Default for RandomXFlag {
    /// Default value for RandomXFlag
    fn default() -> RandomXFlag {
        RandomXFlag::FLAG_DEFAULT
    }
}

#[derive(Debug, Clone, Error)]
/// Custom error enum
pub enum RandomXError {
    #[error("Problem creating the RandomX object: {0}")]
    CreationError(String),
    #[error("Problem with configuration flags: {0}")]
    FlagConfigError(String),
    #[error("Problem with parameters supplied: {0}")]
    ParameterError(String),
    #[error("Failed to convert Int to usize")]
    TryFromIntError(#[from] TryFromIntError),
    #[error("Unknown problem running RandomX: {0}")]
    Other(String),
}

#[derive(Debug)]
pub struct RandomXCacheInner {
    cache_ptr: *mut randomx_cache,
}

impl Drop for RandomXCacheInner {
    /// De-allocates memory for the `cache` object
    fn drop(&mut self) {
        unsafe {
            randomx_release_cache(self.cache_ptr);
        }
    }
}

#[derive(Debug, Clone)]
/// Cache structure
pub struct RandomXCache {
    inner: Arc<RandomXCacheInner>,
}

impl RandomXCache {
    /// Creates a new cache object, allocates memory to the `cache` object and initializes it with
    /// he key value, error on failure.
    ///
    /// `flags` is any combination of the following two flags:
    /// * FLAG_LARGE_PAGES
    /// * FLAG_JIT
    ///
    /// and (optionally) one of the following flags (depending on instruction set supported):
    /// * FLAG_ARGON2_SSSE3
    /// * FLAG_ARGON2_AVX2
    ///
    /// `key` is a sequence of u8 used to initialize SuperScalarHash.
    pub fn new(flags: RandomXFlag, key: &[u8]) -> Result<RandomXCache, RandomXError> {
        if key.is_empty() {
            Err(RandomXError::ParameterError("key is empty".to_string()))
        } else {
            let test = unsafe { randomx_alloc_cache(flags.bits) };
            if test.is_null() {
                Err(RandomXError::CreationError("Could not allocate cache".to_string()))
            } else {
                let inner = RandomXCacheInner { cache_ptr: test };
                let result = RandomXCache { inner: Arc::new(inner) };
                let key_ptr = key.as_ptr() as *mut c_void;
                let key_size = key.len() as usize;
                unsafe {
                    randomx_init_cache(result.inner.cache_ptr, key_ptr, key_size);
                }
                Ok(result)
            }
        }
    }
}

#[derive(Debug)]
pub struct RandomXDatasetInner {
    dataset_ptr: *mut randomx_dataset,
    dataset_count: c_ulong,
    #[allow(dead_code)]
    cache: RandomXCache,
}

impl Drop for RandomXDatasetInner {
    /// De-allocates memory for the `dataset` object.
    fn drop(&mut self) {
        unsafe {
            randomx_release_dataset(self.dataset_ptr);
        }
    }
}

#[derive(Debug, Clone)]
/// Dataset structure
pub struct RandomXDataset {
    inner: Arc<RandomXDatasetInner>,
}

impl RandomXDataset {
    /// Creates a new dataset object, allocates memory to the `dataset` object and initializes it,
    /// error on failure.
    ///
    /// `flags` is one of the following:
    /// * FLAG_DEFAULT
    /// * FLAG_LARGE_PAGES
    ///
    /// `cache` is a cache object.
    ///
    /// `start` is the item number where initialization should start, recommended to pass in 0.
    pub fn new(flags: RandomXFlag, cache: RandomXCache, start: c_ulong) -> Result<RandomXDataset, RandomXError> {
        let count = u64::from(RANDOMX_DATASET_ITEM_SIZE - 1) - start;
        let test = unsafe { randomx_alloc_dataset(flags.bits) };
        if test.is_null() {
            Err(RandomXError::CreationError("Could not allocate dataset".to_string()))
        } else {
            let inner = RandomXDatasetInner {
                dataset_ptr: test,
                dataset_count: count,
                cache,
            };
            let result = RandomXDataset { inner: Arc::new(inner) };
            let item_count = result
                .count()
                .map_err(|err| RandomXError::CreationError(format!("Could not get dataset count:{}", err)))?;
            // Mirror the assert checks inside randomx_init_dataset call
            if (start < item_count && count <= item_count) || (start + item_count <= count) {
                unsafe {
                    randomx_init_dataset(
                        result.inner.dataset_ptr,
                        result.inner.cache.inner.cache_ptr,
                        start as c_ulong,
                        count as c_ulong,
                    );
                }
                Ok(result)
            } else {
                let reason = format!(
                    "Dataset `start` or `count` was out of bounds: start: {}, count: {}, actual count: {}",
                    start, count, item_count
                );
                Err(RandomXError::CreationError(reason))
            }
        }
    }

    /// Returns the number of items in the `dataset` or an error on failure.
    pub fn count(&self) -> Result<u64, RandomXError> {
        match unsafe { randomx_dataset_item_count() } {
            0 => Err(RandomXError::Other("Dataset item count was 0".to_string())),
            x => Ok(x as u64),
        }
    }

    /// Returns the values of the internal memory buffer of the `dataset` or an error on failure.
    pub fn get_data(&self) -> Result<Vec<u8>, RandomXError> {
        let memory = unsafe { randomx_get_dataset_memory(self.inner.dataset_ptr) };
        if memory.is_null() {
            Err(RandomXError::Other("Could not get dataset memory".into()))
        } else {
            let count = usize::try_from(self.inner.dataset_count)?;
            let mut result: Vec<u8> = vec![0u8; count];
            let n = usize::try_from(self.inner.dataset_count)?;
            unsafe {
                libc::memcpy(result.as_mut_ptr() as *mut c_void, memory, n);
            }
            Ok(result)
        }
    }
}

#[derive(Debug)]
/// VM structure
pub struct RandomXVM {
    flags: RandomXFlag,
    vm: *mut randomx_vm,
    linked_cache: Option<RandomXCache>,
    linked_dataset: Option<RandomXDataset>,
}

impl Drop for RandomXVM {
    /// De-allocates memory for the `VM` object.
    fn drop(&mut self) {
        unsafe {
            randomx_destroy_vm(self.vm);
        }
    }
}

impl RandomXVM {
    /// Creates a new `VM` and initializes it, error on failure.
    ///
    /// `flags` is any combination of the following 5 flags:
    /// * FLAG_LARGE_PAGES
    /// * FLAG_HARD_AES
    /// * FLAG_FULL_MEM
    /// * FLAG_JIT
    /// * FLAG_SECURE
    ///
    /// Or
    ///
    /// * FLAG_DEFAULT
    ///
    /// `cache` is a cache object, optional if FLAG_FULL_MEM is set.
    ///
    /// `dataset` is a dataset object, optional if FLAG_FULL_MEM is not set.
    pub fn new(
        flags: RandomXFlag,
        cache: Option<RandomXCache>,
        dataset: Option<RandomXDataset>,
    ) -> Result<RandomXVM, RandomXError> {
        let is_full_mem = flags.contains(RandomXFlag::FLAG_FULL_MEM);
        match (cache, dataset) {
            (None, None) => Err(RandomXError::CreationError("Failed to allocate VM".to_string())),
            (None, _) if !is_full_mem => Err(RandomXError::FlagConfigError(
                "No cache and FLAG_FULL_MEM not set".to_string(),
            )),
            (_, None) if is_full_mem => Err(RandomXError::FlagConfigError(
                "No dataset and FLAG_FULL_MEM set".to_string(),
            )),
            (cache, dataset) => {
                let cache_ptr = cache
                    .as_ref()
                    .map(|stash| stash.inner.cache_ptr)
                    .unwrap_or_else(ptr::null_mut);
                let dataset_ptr = dataset
                    .as_ref()
                    .map(|data| data.inner.dataset_ptr)
                    .unwrap_or_else(ptr::null_mut);
                let vm = unsafe { randomx_create_vm(flags.bits, cache_ptr, dataset_ptr) };
                Ok(RandomXVM {
                    vm,
                    flags,
                    linked_cache: cache,
                    linked_dataset: dataset,
                })
            },
        }
    }

    /// Re-initializes the `VM` with a new cache that was initialised without
    /// RandomXFlag::FLAG_FULL_MEM.
    pub fn reinit_cache(&mut self, cache: RandomXCache) -> Result<(), RandomXError> {
        if self.flags.contains(RandomXFlag::FLAG_FULL_MEM) {
            Err(RandomXError::FlagConfigError(
                "Cannot reinit cache with FLAG_FULL_MEM set".to_string(),
            ))
        } else {
            unsafe {
                randomx_vm_set_cache(self.vm, cache.inner.cache_ptr);
            }
            self.linked_cache = Some(cache);
            Ok(())
        }
    }

    /// Re-initializes the `VM` with a new dataset that was initialised with
    /// RandomXFlag::FLAG_FULL_MEM.
    pub fn reinit_dataset(&mut self, dataset: RandomXDataset) -> Result<(), RandomXError> {
        if self.flags.contains(RandomXFlag::FLAG_FULL_MEM) {
            unsafe {
                randomx_vm_set_dataset(self.vm, dataset.inner.dataset_ptr);
            }
            self.linked_dataset = Some(dataset);
            Ok(())
        } else {
            Err(RandomXError::FlagConfigError(
                "Cannot reinit dataset without FLAG_FULL_MEM set".to_string(),
            ))
        }
    }

    /// Calculates a RandomX hash value and returns it, error on failure.
    ///
    /// `input` is a sequence of u8 to be hashed.
    pub fn calculate_hash(&self, input: &[u8]) -> Result<Vec<u8>, RandomXError> {
        if input.is_empty() {
            Err(RandomXError::ParameterError("input was empty".to_string()))
        } else {
            let size_input = input.len() as usize;
            let input_ptr = input.as_ptr() as *mut c_void;
            let arr = [0; RANDOMX_HASH_SIZE as usize];
            let output_ptr = arr.as_ptr() as *mut c_void;
            unsafe {
                randomx_calculate_hash(self.vm, input_ptr, size_input, output_ptr);
            }
            // if this failed, arr should still be empty
            if arr == [0; RANDOMX_HASH_SIZE as usize] {
                Err(RandomXError::Other("RandomX calculated hash was empty".to_string()))
            } else {
                let result = arr.to_vec();
                Ok(result)
            }
        }
    }

    /// Calculates hashes from a set of inputs.
    ///
    /// `input` is an array of a sequence of u8 to be hashed.
    #[allow(clippy::needless_range_loop)] // Range loop is not only for indexing `input`
    pub fn calculate_hash_set(&self, input: &[&[u8]]) -> Result<Vec<Vec<u8>>, RandomXError> {
        if input.is_empty() {
            // Empty set
            return Err(RandomXError::ParameterError("input was empty".to_string()));
        }

        let mut result = Vec::new();
        // For single input
        if input.len() == 1 {
            let hash = self.calculate_hash(input[0])?;
            result.push(hash);
            return Ok(result);
        }

        // For multiple inputs
        let mut output_ptr: *mut c_void = ptr::null_mut();
        let arr = [0; RANDOMX_HASH_SIZE as usize];

        // Not len() as last iteration assigns final hash
        let iterations = input.len() + 1;
        for i in 0..iterations {
            if i == iterations - 1 {
                // For last iteration
                unsafe {
                    randomx_calculate_hash_last(self.vm, output_ptr);
                }
            } else {
                if input[i].is_empty() {
                    // Stop calculations
                    if arr != [0; RANDOMX_HASH_SIZE as usize] {
                        // Complete what was started
                        unsafe {
                            randomx_calculate_hash_last(self.vm, output_ptr);
                        }
                    }
                    return Err(RandomXError::ParameterError("input was empty".to_string()));
                };
                let size_input = input[i].len() as usize;
                let input_ptr = input[i].as_ptr() as *mut c_void;
                output_ptr = arr.as_ptr() as *mut c_void;
                if i == 0 {
                    // For first iteration
                    unsafe {
                        randomx_calculate_hash_first(self.vm, input_ptr, size_input);
                    }
                } else {
                    unsafe {
                        // For every other iteration
                        randomx_calculate_hash_next(self.vm, input_ptr, size_input, output_ptr);
                    }
                }
            }

            if i != 0 {
                // First hash is only available in 2nd iteration
                if arr == [0; RANDOMX_HASH_SIZE as usize] {
                    return Err(RandomXError::Other("RandomX hash was zero".to_string()));
                }
                let output: Vec<u8> = arr.to_vec();
                result.push(output);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{RandomXCache, RandomXDataset, RandomXFlag, RandomXVM};

    #[test]
    fn lib_alloc_cache() {
        let flags = RandomXFlag::default();
        let key = "Key";
        let cache = RandomXCache::new(flags, key.as_bytes());
        if let Err(i) = cache {
            panic!("Failed to allocate cache, {}", i);
        }
        drop(cache);
    }

    #[test]
    fn lib_alloc_dataset() {
        let flags = RandomXFlag::default();
        let key = "Key";
        let cache = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let dataset = RandomXDataset::new(flags, cache.clone(), 0);
        if let Err(i) = dataset {
            panic!("Failed to allocate dataset, {}", i);
        }
        drop(dataset);
        drop(cache);
    }

    #[test]
    fn lib_alloc_vm() {
        let flags = RandomXFlag::default();
        let key = "Key";
        let cache = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let mut vm = RandomXVM::new(flags, Some(cache.clone()), None);
        if let Err(i) = vm {
            panic!("Failed to allocate vm, {}", i);
        }
        drop(vm);
        let dataset = RandomXDataset::new(flags, cache.clone(), 0).unwrap();
        vm = RandomXVM::new(flags, Some(cache.clone()), Some(dataset.clone()));
        if let Err(i) = vm {
            panic!("Failed to allocate vm, {}", i);
        }
        drop(dataset);
        drop(cache);
        drop(vm);
    }

    #[test]
    fn lib_dataset_memory() {
        let flags = RandomXFlag::default();
        let key = "Key";
        let cache = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let dataset = RandomXDataset::new(flags, cache.clone(), 0).unwrap();
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
        let flags = RandomXFlag::get_recommended_flags();
        let flags2 = flags | RandomXFlag::FLAG_FULL_MEM;
        let key = "Key";
        let input = "Input";
        let cache1 = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let mut vm1 = RandomXVM::new(flags, Some(cache1.clone()), None).unwrap();
        let hash1 = vm1.calculate_hash(input.as_bytes()).expect("no data");
        let vec = vec![0u8; hash1.len() as usize];
        assert_ne!(hash1, vec);
        let reinit_cache = vm1.reinit_cache(cache1.clone());
        assert_eq!(reinit_cache.is_ok(), true);
        let hash2 = vm1.calculate_hash(input.as_bytes()).expect("no data");
        assert_ne!(hash2, vec);
        assert_eq!(hash1, hash2);

        let cache2 = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let vm2 = RandomXVM::new(flags, Some(cache2.clone()), None).unwrap();
        let hash3 = vm2.calculate_hash(input.as_bytes()).expect("no data");
        assert_eq!(hash2, hash3);

        let cache3 = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let dataset3 = RandomXDataset::new(flags, cache3.clone(), 0).unwrap();
        let mut vm3 = RandomXVM::new(flags2, None, Some(dataset3.clone())).unwrap();
        let hash4 = vm3.calculate_hash(input.as_bytes()).expect("no data");
        assert_ne!(hash3, vec);
        let reinit_dataset = vm3.reinit_dataset(dataset3.clone());
        assert_eq!(reinit_dataset.is_ok(), true);
        let hash5 = vm3.calculate_hash(input.as_bytes()).expect("no data");
        assert_ne!(hash4, vec);
        assert_eq!(hash4, hash5);

        let cache4 = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let dataset4 = RandomXDataset::new(flags, cache4.clone(), 0).unwrap();
        let vm4 = RandomXVM::new(flags2, Some(cache4.clone()), Some(dataset4.clone())).unwrap();
        let hash6 = vm3.calculate_hash(input.as_bytes()).expect("no data");
        assert_eq!(hash5, hash6);

        drop(dataset3);
        drop(dataset4);
        drop(cache1);
        drop(cache2);
        drop(cache3);
        drop(vm1);
        drop(vm2);
        drop(vm3);
        drop(vm4);
    }

    #[test]
    fn lib_calculate_hash_set() {
        let flags = RandomXFlag::default();
        let key = "Key";
        let mut inputs = Vec::new();
        inputs.push("Input".as_bytes());
        inputs.push("Input 2".as_bytes());
        inputs.push("Inputs 3".as_bytes());
        let cache = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let vm = RandomXVM::new(flags, Some(cache.clone()), None).unwrap();
        let hashes = vm.calculate_hash_set(inputs.as_slice()).expect("no data");
        assert_eq!(inputs.len(), hashes.len());
        let mut prev_hash = Vec::new();
        let mut i = 0;
        for hash in hashes {
            let vec = vec![0u8; hash.len() as usize];
            assert_ne!(hash, vec);
            assert_ne!(hash, prev_hash);
            let compare = vm.calculate_hash(inputs[i]).unwrap(); // sanity check
            assert_eq!(hash, compare);
            prev_hash = hash;
            i += 1;
        }
        drop(cache);
        drop(vm);
    }

    #[test]
    fn lib_calculate_hash_is_consistent() {
        let flags = RandomXFlag::get_recommended_flags();
        let key = "Key";
        let input = "Input";
        let cache = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let dataset = RandomXDataset::new(flags, cache.clone(), 0).unwrap();
        let vm = RandomXVM::new(flags, Some(cache.clone()), Some(dataset.clone())).unwrap();
        let hash = vm.calculate_hash(input.as_bytes()).expect("no data");
        assert_eq!(hash, [
            114, 81, 192, 5, 165, 242, 107, 100, 184, 77, 37, 129, 52, 203, 217, 227, 65, 83, 215, 213, 59, 71, 32,
            172, 253, 155, 204, 111, 183, 213, 157, 155
        ]);
        drop(vm);
        drop(dataset);
        drop(cache);

        let cache1 = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let dataset1 = RandomXDataset::new(flags, cache1.clone(), 0).unwrap();
        let vm1 = RandomXVM::new(flags, Some(cache1.clone()), Some(dataset1.clone())).unwrap();
        let hash1 = vm1.calculate_hash(input.as_bytes()).expect("no data");
        assert_eq!(hash1, [
            114, 81, 192, 5, 165, 242, 107, 100, 184, 77, 37, 129, 52, 203, 217, 227, 65, 83, 215, 213, 59, 71, 32,
            172, 253, 155, 204, 111, 183, 213, 157, 155
        ]);
        drop(vm1);
        drop(dataset1);
        drop(cache1);
    }

    #[test]
    fn lib_check_cache_and_dataset_lifetimes() {
        let flags = RandomXFlag::get_recommended_flags();
        let key = "Key";
        let input = "Input";
        let cache = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let dataset = RandomXDataset::new(flags, cache.clone(), 0).unwrap();
        let vm = RandomXVM::new(flags, Some(cache.clone()), Some(dataset.clone())).unwrap();
        drop(dataset);
        drop(cache);
        let hash = vm.calculate_hash(input.as_bytes()).expect("no data");
        assert_eq!(hash, [
            114, 81, 192, 5, 165, 242, 107, 100, 184, 77, 37, 129, 52, 203, 217, 227, 65, 83, 215, 213, 59, 71, 32,
            172, 253, 155, 204, 111, 183, 213, 157, 155
        ]);
        drop(vm);

        let cache1 = RandomXCache::new(flags, key.as_bytes()).unwrap();
        let dataset1 = RandomXDataset::new(flags, cache1.clone(), 0).unwrap();
        let vm1 = RandomXVM::new(flags, Some(cache1.clone()), Some(dataset1.clone())).unwrap();
        drop(dataset1);
        drop(cache1);
        let hash1 = vm1.calculate_hash(input.as_bytes()).expect("no data");
        assert_eq!(hash1, [
            114, 81, 192, 5, 165, 242, 107, 100, 184, 77, 37, 129, 52, 203, 217, 227, 65, 83, 215, 213, 59, 71, 32,
            172, 253, 155, 204, 111, 183, 213, 157, 155
        ]);
        drop(vm1);
    }
}
