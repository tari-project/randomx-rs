#![no_main]

use libfuzzer_sys::fuzz_target;
use randomx_rs::test_utils::fuzz_randomx_vm_calculate_hash_with_cache_and_dataset;

fuzz_target!(|data: &[u8]| {
    fuzz_randomx_vm_calculate_hash_with_cache_and_dataset(data.to_vec());
});
