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
mod bindings;

use bindings::{
    randomx_alloc_cache, randomx_alloc_dataset, randomx_cache, randomx_calculate_hash,
    randomx_create_vm, randomx_dataset, randomx_dataset_item_count, randomx_destroy_vm,
    randomx_get_dataset_memory, randomx_init_cache, randomx_init_dataset, randomx_release_cache,
    randomx_release_dataset, randomx_vm, randomx_vm_set_cache, randomx_vm_set_dataset,
    RANDOMX_DATASET_ITEM_SIZE, RANDOMX_HASH_SIZE,
};
use derive_error::Error;


use std::mem;
use std::os::raw::{c_char, c_uint, c_ulong, c_void};
use std::ptr;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum RandomXFlag {
    FlagDefault,
    FlagLargePages,
    FlagHardAES,
    FlagFullMem,
    FlagJIT,
    FlagSecure,
}

impl RandomXFlag {
    pub fn value(self) -> u32 {
        match self {
            RandomXFlag::FlagDefault => 0,
            RandomXFlag::FlagLargePages => 1,
            RandomXFlag::FlagHardAES => 2,
            RandomXFlag::FlagFullMem => 4,
            RandomXFlag::FlagJIT => 8,
            RandomXFlag::FlagSecure => 16,
        }
    }
}

impl PartialEq for RandomXFlag {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

#[derive(Debug, Clone, Error)]
pub enum RandomXError {
    // Problem creating the randomX VM
    CreationError,
    // Problem running Random X
    Other,
}

#[derive(Debug)]
pub struct RandomXCache {
    cache: *mut randomx_cache,
}

impl Drop for RandomXCache {
    fn drop(&mut self) {
        unsafe {
            randomx_release_cache(self.cache);
        }
    }
}

impl RandomXCache {
    pub fn new(flags: Vec<RandomXFlag>, key: &str) -> Result<RandomXCache, RandomXError> {
        if key.len() == 0 {
            return Err(RandomXError::CreationError);
        };
        let mut flag: c_uint = RandomXFlag::FlagDefault.value();
        for f in flags {
            if f == RandomXFlag::FlagJIT || f == RandomXFlag::FlagLargePages {
                flag |= f.value();
            }
        }
        let test = unsafe { randomx_alloc_cache(flag) };
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
pub struct RandomXDataset {
    dataset: *mut randomx_dataset,
    dataset_start: c_ulong,
    dataset_count: c_ulong,
}

impl Drop for RandomXDataset {
    fn drop(&mut self) {
        unsafe {
            randomx_release_dataset(self.dataset);
        }
    }
}

impl RandomXDataset {
    pub fn new(
        flags: Vec<RandomXFlag>,
        cache: &RandomXCache,
        start: c_ulong,
    ) -> Result<RandomXDataset, RandomXError> {
        let count = c_ulong::from(RANDOMX_DATASET_ITEM_SIZE - 1) - start;

        let mut flag: c_uint = RandomXFlag::FlagDefault.value();
        for f in flags {
            if f == RandomXFlag::FlagLargePages {
                flag |= f.value();
            }
        }
        let test = unsafe { randomx_alloc_dataset(flag) };
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
            if !((start < item_count && count <= item_count) || (start + item_count <= count)) {
                return Err(RandomXError::CreationError);
            }
            unsafe {
                //no way to check if this fails, c code does not return anything
                randomx_init_dataset(result.dataset, cache.cache, start, count);
            }
            Ok(result)
        }
    }

    pub fn count(&self) -> Result<u64, RandomXError> {
        match unsafe { randomx_dataset_item_count() } {
            0 => Err(RandomXError::Other),
            x => Ok(x),
        }
    }

    pub fn get_data(&self) -> Result<Vec<u8>, RandomXError> {
        let memory = unsafe { randomx_get_dataset_memory(self.dataset) };
        if memory.is_null() {
            return Err(RandomXError::Other);
        }
        let mut result = Vec::new();
        unsafe {
            for i in self.dataset_start..self.dataset_count {
                result.push(memory.offset(i as isize) as u8);
            }
        }
        Ok(result)
    }
}

#[derive(Debug)]
pub struct RandomXVM {
    vm: *mut randomx_vm,
}

impl Drop for RandomXVM {
    fn drop(&mut self) {
        unsafe {
            randomx_destroy_vm(self.vm);
        }
    }
}

impl RandomXVM {
    pub fn new(
        flags: Vec<RandomXFlag>,
        cache: &RandomXCache,
        dataset: Option<&RandomXDataset>,
    ) -> Result<RandomXVM, RandomXError> {
        let mut flag: c_uint = RandomXFlag::FlagDefault.value();
        for f in flags {
            flag |= f.value();
        }
        let test: *mut randomx_vm;
        match dataset {
            Some(data) => unsafe { test = randomx_create_vm(flag, cache.cache, data.dataset) },
            None => unsafe { test = randomx_create_vm(flag, cache.cache, ptr::null_mut()) },
        }
        if test.is_null() {
            return Err(RandomXError::CreationError);
        }
        let result = RandomXVM { vm: test };
        Ok(result)
    }

    pub fn reinit_cache(&self, cache: &RandomXCache) {
        //no way to check if this fails, c code does not return anything
        unsafe {
            randomx_vm_set_cache(self.vm, cache.cache);
        }
    }

    pub fn reinit_dataset(&self, dataset: &RandomXDataset) {
        //no way to check if this fails, c code does not return anything
        unsafe {
            randomx_vm_set_dataset(self.vm, dataset.dataset);
        }
    }

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
        let mut result = Vec::new();

        for i in 0..RANDOMX_HASH_SIZE {
            result.push(arr[i as usize] as u8);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lib_alloc_cache() {
        let mut flags = Vec::new();
        let key = "Key";
        flags.push(RandomXFlag::FlagDefault);
        let cache = RandomXCache::new(flags.clone(), key);
        if let Err(i) = cache {
            panic!(format!("Failed to allocate cache, {}", i));
        }
    }

    #[test]
    fn lib_alloc_dataset() {
        let mut flags = Vec::new();
        let key = "Key";
        flags.push(RandomXFlag::FlagDefault);
        let cache = RandomXCache::new(flags.clone(), key).unwrap();
        let dataset = RandomXDataset::new(flags.clone(), &cache, 0);
        if let Err(i) = dataset {
            panic!(format!("Failed to allocate dataset, {}", i));
        }
    }

    #[test]
    fn lib_alloc_vm() {
        let mut flags = Vec::new();
        let key = "Key";
        flags.push(RandomXFlag::FlagDefault);
        let cache = RandomXCache::new(flags.clone(), key).unwrap();
        let mut vm = RandomXVM::new(flags.clone(), &cache, None);
        if let Err(i) = vm {
            panic!(format!("Failed to allocate vm, {}", i));
        }
        let dataset = RandomXDataset::new(flags.clone(), &cache, 0).unwrap();
        vm = RandomXVM::new(flags, &cache, Some(&dataset));
        if let Err(i) = vm {
            panic!(format!("Failed to allocate vm, {}", i));
        }
    }

    #[test]
    fn lib_dataset_memory() {
        let mut flags = Vec::new();
        let key = "Key";
        flags.push(RandomXFlag::FlagDefault);
        let cache = RandomXCache::new(flags.clone(), key).unwrap();
        let dataset = RandomXDataset::new(flags.clone(), &cache, 0).unwrap();
        let memory = dataset.get_data().expect("no data");
        let mut vec: Vec<u8> = Vec::new();
        for i in 0..memory.len() - 1 {
            vec.push(i as u8);
        }
        assert_ne!(memory, vec);
    }

    #[test]
    fn lib_calculate_hash() {
        let mut flags = Vec::new();
        let key = "Key";
        let input = "Input";
        flags.push(RandomXFlag::FlagDefault);
        let cache = RandomXCache::new(flags.clone(), key).unwrap();
        let vm = RandomXVM::new(flags.clone(), &cache, None).unwrap();
        let hash = vm.calculate_hash(input).expect("no data");
        let mut vec: Vec<u8> = Vec::new();
        for i in 0..hash.len() - 1 {
            vec.push(i as u8);
        }
        assert_ne!(hash, vec);
        vm.reinit_cache(&cache);
        let dataset = RandomXDataset::new(flags.clone(), &cache, 0).unwrap();
        vm.reinit_dataset(&dataset);
        let hash = vm.calculate_hash(input).expect("no data");
        vec = Vec::new();
        for i in 0..hash.len() - 1 {
            vec.push(i as u8);
        }
        assert_ne!(hash, vec);
    }
}
