#![no_main]

use libfuzzer_sys::fuzz_target;
use randomx_rs::test_utils::fuzz_randomx_alloc_cache;

fuzz_target!(|data: &[u8]| {
    fuzz_randomx_alloc_cache(data.to_vec());
});
