// Copyright 2024 The Tari Project
// SPDX-License-Identifier: BSD-3-Clause

#![no_main]

use libfuzzer_sys::fuzz_target;
use randomx_rs::test_utils::fuzz_randomx_alloc_cache;

fuzz_target!(|data: &[u8]| {
    fuzz_randomx_alloc_cache(data.to_vec());
});
