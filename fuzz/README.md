# Fuzzing randomx-rs

See https://rust-fuzz.github.io/book/cargo-fuzz.html for more information on fuzzing with cargo-fuzz.
Install `cargo-fuzz` as per [installation instructions](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html).


**Note:** Fuzzing is not supported on Windows yet.

To get a list of fuzz targets, from a terminal in the project root, run
```
cargo fuzz list
```

To run a fuzz test, from a terminal in the project root, run
```
cargo +nightly fuzz run --release <fuzz_target_name>
```
To run fuzz tests involving a cache and dataset, on error `libFuzzer: out-of-memory (malloc(2181038016))`, pass 
`-- -rss_limit_mb=<ram_upper_limit>` as argument to allow using more than 2 GB of RAM - 3GB recommended.
```
cargo +nightly fuzz run --release <fuzz_target_name> -- -rss_limit_mb=3221225472
```
