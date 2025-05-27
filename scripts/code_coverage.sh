#!/bin/bash
# Prerequisites
# 1. You need LLVM-COV tools:
# $ rustup component add llvm-tools-preview
# 2. and Rust wrappers for llvm-cov:
# $ cargo install cargo-binutils
# 3. The rust name demangler
# $ cargo install rustfilt
# 4. jq
# 5. genhtml
# $ sudo apt install lcov

NAME="randomx-rs"
RUSTFLAGS="-Zinstrument-coverage"
LLVM_PROFILE_FILE="./cov_raw/$NAME-%m.profraw"

get_binaries() {
    files=$(
        RUSTFLAGS=$RUSTFLAGS cargo test --tests --no-run --message-format=json |
            jq -r "select(.profile.test == true) | .filenames[]" |
            grep -v dSYM -
    )
    files=("${files[@]/#/-object }")
}

get_binaries

# echo "files: $files"

# Remove old coverage files
rm cov_raw/*profraw cov_raw/$NAME.profdata cov_raw/$NAME.lcov cov_raw/$NAME.txt

RUSTFLAGS=$RUSTFLAGS LLVM_PROFILE_FILE=$LLVM_PROFILE_FILE cargo test --tests

cargo profdata -- \
    merge -sparse ./cov_raw/$NAME*.profraw -o ./cov_raw/$NAME.profdata

cargo cov -- \
    export \
    --Xdemangler=rustfilt \
    --format=lcov \
    --show-branch-summary \
    --show-instantiation-summary \
    --show-region-summary \
    --ignore-filename-regex='\.cargo' \
    --ignore-filename-regex="rustc" \
    --ignore-filename-regex="\.git" \
    --instr-profile=cov_raw/$NAME.profdata \
    $files \
    >cov_raw/$NAME.lcov

cargo cov -- \
    show \
    --Xdemangler=rustfilt \
    --show-branch-summary \
    --show-instantiation-summary \
    --show-region-summary \
    --ignore-filename-regex='\.cargo' \
    --ignore-filename-regex="rustc" \
    --ignore-filename-regex="\.git" \
    --instr-profile=cov_raw/$NAME.profdata \
    $files \
    >cov_raw/$NAME.txt

if [ -z ${SKIP_HTML+x} ]; then
    genhtml -o coverage_report cov_raw/$NAME.lcov
else
    echo "Skipping html generation"
fi
# open coverage_report/src/index.html
