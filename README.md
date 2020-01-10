# randomx-rs
Rust bindings to the RandomX proof-of-work (Pow) system

## Build Dependencies
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

Install [Build Tools for Visual Studio 2019](
https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)
