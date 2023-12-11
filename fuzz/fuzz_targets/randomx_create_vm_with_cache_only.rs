#![no_main]

use libfuzzer_sys::fuzz_target;
use randomx_rs::test_utils::fuzz_randomx_create_vm_with_cache_only;

fuzz_target!(|data: &[u8]| {
    fuzz_randomx_create_vm_with_cache_only(data.to_vec());
});
