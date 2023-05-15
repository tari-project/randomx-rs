![CI](https://github.com/tari-project/randomx-rs/actions/workflows/ci.yml/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/delta1/randomx-rs/badge.svg?branch=cov-temp)](https://coveralls.io/github/delta1/randomx-rs?branch=cov-temp)

# RandomX-rs

> Rust bindings to the RandomX proof-of-work (Pow) system

## Build Dependencies

This repo makes use of git submodules.

The first time you compile, or perhaps after a big update after a `git pull`, you need to update the submodules:

```bash
git submodule init
git submodule update
```

If you see an error like

```
fatal: Needed a single revision
Unable to find current revision in submodule path 'RandomX'
```

you might want to see if there is a `RandomX` folder in the source tree. (On case insensitive systems, like OsX and Windows, it might
even be `randomx`). Deleting this folder and repeating the commands above should resolve the issue.

### Mac

Install [XCode](https://apps.apple.com/za/app/xcode/id497799835?mt=12) and then the XCode Command Line Tools with the following command

```
xcode-select --install
```

For macOS Mojave additional headers need to be installed, run

```
open /Library/Developer/CommandLineTools/Packages/macOS_SDK_headers_for_macOS_10.14.pkg
```

and follow the prompts

Install Brew

```
/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
```

Run the following to install needed bottles

```
brew install git
brew install cmake
```

### Linux

Run the following to install dependencies

```
apt-get install git cmake libc++-dev libc++abi-dev
```

### Windows

Install [Git](https://git-scm.com/download/win)

Install [CMake](https://cmake.org/download/)

Install [Build Tools for Visual Studio 2019](https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)

### Android

To build using the Android NDK the `ANDROID_SDK_ROOT` environment variable needs to be set. Other variables are optional as they have defaults. Example build command for ARM64:
```
ANDROID_SDK_ROOT=/home/user/Android/Sdk \
ANDROID_PLATFORM=android-25 \
ANDROID_CMAKE=/home/user/Android/Sdk/cmake/3.22.1/bin/cmake \
ANDROID_CMAKE_TOOLCHAIN=/home/user/Android/Sdk/ndk/22.1.7171670/build/cmake/android.toolchain.cmake \
cargo build --target=aarch64-linux-android
```

# Troubleshooting

## Mac/OSX

If you're experiencing linker issues, or messages like

`cstdint:153:8: error: no member named 'int8_t' in the global namespace`

then you might have multiple conflicting versions of clang installed.

Try:

- Does `which cc` report more than one binary? If so, uninstalling one of the clang compilers might help.
- Upgrading cmake. `brew uninstall cmake && brew install cmake`
- `cargo clean`

On Apple ARM64 hardware and newer XCode releases, RandomX might fail the `randomx-tests`.
```
[83] Hash test 1e (interpreter)               ... PASSED
[84] Hash test 2a (compiler)                  ... Assertion failed: (equalsHex(hash, "639183aae1bf4c9a35884cb46b09cad9175f04efd7684e7262a0ac1c2f0b4e3f")), function operator(), file tests.cpp, line 966.
zsh: abort      ./randomx-tests
```
or
```
[88] Hash test 2e (compiler)                  ... PASSED
[89] Cache initialization: SSSE3              ... SKIPPED
[90] Cache initialization: AVX2               ... SKIPPED
[91] Hash batch test                          ... Assertion failed: (equalsHex(hash3, "c36d4ed4191e617309867ed66a443be4075014e2b061bcdaf9ce7b721d2b77a8")), function operator(), file tests.cpp, line 1074.
zsh: abort      ./randomx-tests
```
 Building using an older SDK might help. Find location of current SDKs with `xcrun --show-sdk-path`, then for example:
```bash
export RANDOMX_RS_CMAKE_OSX_SYSROOT="/Library/Developer/CommandLineTools/SDKs/MacOSX12.3.sdk"
cargo build
```
Quick test with built binaries
```bash
find target -name randomx-tests -exec {} \;
```
