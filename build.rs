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
use std::{
    env,
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    process::Command,
};

#[allow(clippy::too_many_lines)]
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let project_dir = Path::new(&out_dir);
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let repo_dir = PathBuf::from(env::var("RANDOMX_DIR").unwrap_or_else(|_| format!("{}/RandomX", &cargo_dir)));
    let build_dir = &project_dir.join("randomx_build");

    env::set_current_dir(Path::new(&repo_dir)).unwrap(); // change current path to repo for dependency build
    match fs::create_dir_all(build_dir) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => (),
            _ => panic!("{}", e),
        },
    }
    env::set_current_dir(build_dir).unwrap();

    let host = env::var("HOST").unwrap();
    // println!("host: {}", host);
    let target = env::var("TARGET").unwrap();
    // println!("target: {}", target);
    if host.contains("windows") && target.contains("windows-msvc") {
        let c = Command::new("cmake")
            .arg("-G")
            .arg("Visual Studio 16 2019")
            .arg(repo_dir.to_str().unwrap())
            .output()
            .expect("failed to execute CMake");
        println!("status: {}", c.status);
        std::io::stdout().write_all(&c.stdout).unwrap();
        std::io::stderr().write_all(&c.stderr).unwrap();
        assert!(c.status.success());

        let m = Command::new("cmake")
            .arg("--build")
            .arg(".")
            .arg("--config")
            .arg("Release")
            .output()
            .expect("failed to execute Make");
        println!("status: {}", m.status);
        std::io::stdout().write_all(&m.stdout).unwrap();
        std::io::stderr().write_all(&m.stderr).unwrap();
        assert!(m.status.success());
    } else if target.contains("aarch64-unknown-linux-gnu") {
        let c = Command::new("cmake")
            .arg("-D")
            .arg("ARCH=arm64")
            .arg("-D")
            .arg("ARCH_ID=aarch64")
            .arg("-D")
            .arg("CMAKE_CROSSCOMPILING=true")
            .arg("-D")
            .arg("CMAKE_SYSTEM_PROCESSOR=aarch64")
            .arg("-D")
            .arg("CMAKE_C_COMPILER=/usr/bin/aarch64-linux-gnu-gcc")
            .arg("-D")
            .arg("CMAKE_CXX_COMPILER=/usr/bin/aarch64-linux-gnu-g++")
            .arg(repo_dir.to_str().unwrap())
            .output()
            .expect("failed to execute CMake");
        println!("status: {}", c.status);
        std::io::stdout().write_all(&c.stdout).unwrap();
        std::io::stderr().write_all(&c.stderr).unwrap();
        assert!(c.status.success());

        let m = Command::new("cmake")
            .arg("--build")
            .arg(".")
            .arg("--config")
            .arg("Release")
            .output()
            .expect("failed to execute Make");
        println!("status: {}", m.status);
        std::io::stdout().write_all(&m.stdout).unwrap();
        std::io::stderr().write_all(&m.stderr).unwrap();
        assert!(m.status.success());
    } else if target.contains("linux-android") {
        let android_abi = {
            if target.contains("aarch64") {
                "arm64-v8a"
            } else if target.contains("x86_64") {
                "x86_64"
            } else if target.contains("armv7") {
                "armeabi-v7a"
            } else if target.contains("i686") {
                "x86"
            } else {
                panic!("unknown Android ABI")
            }
        };

        let android_sdk = env::var("ANDROID_SDK_ROOT").expect("ANDROID_SDK_ROOT variable not set");

        let android_platform = env::var("ANDROID_PLATFORM").unwrap_or_else(|_| "android-26".to_owned());
        let android_cmake = env::var("ANDROID_CMAKE").unwrap_or(android_sdk.clone() + "/cmake/3.22.1/bin/cmake");
        let android_toolchain = env::var("ANDROID_CMAKE_TOOLCHAIN")
            .unwrap_or(android_sdk + "/ndk/22.1.7171670/build/cmake/android.toolchain.cmake");

        let c = Command::new(android_cmake)
            .arg("-D")
            .arg("CMAKE_TOOLCHAIN_FILE=".to_owned() + &android_toolchain)
            .arg("-D")
            .arg("ANDROID_ABI=".to_owned() + android_abi)
            .arg("-D")
            .arg("ANDROID_PLATFORM=".to_owned() + &android_platform)
            .arg(repo_dir.to_str().unwrap())
            .output()
            .expect("failed to execute CMake");

        println!("status: {}", c.status);
        std::io::stdout().write_all(&c.stdout).unwrap();
        std::io::stderr().write_all(&c.stderr).unwrap();
        assert!(c.status.success());

        let m = Command::new("make").output().expect("failed to execute Make");
        println!("status: {}", m.status);
        std::io::stdout().write_all(&m.stdout).unwrap();
        std::io::stderr().write_all(&m.stderr).unwrap();
        assert!(m.status.success());
    } else if target.contains("aarch64-apple-darwin") {
        let mut c = Command::new("cmake");
        c.arg("-D")
            .arg("ARCH=arm64")
            .arg("-D")
            .arg("ARCH_ID=aarch64")
            .arg("-D")
            .arg("CMAKE_CROSSCOMPILING=true")
            .arg("-D")
            .arg("CMAKE_SYSTEM_PROCESSOR=aarch64")
            .arg("-D")
            .arg("CMAKE_C_FLAGS='-arch arm64'")
            .arg("-D")
            .arg("CMAKE_CXX_FLAGS='-arch arm64'");
        if let Ok(env) = env::var("RANDOMX_RS_CMAKE_OSX_SYSROOT") {
            c.arg("-D").arg("CMAKE_OSX_SYSROOT=".to_owned() + env.as_str());
        }
        let output = c
            .arg(repo_dir.to_str().unwrap())
            .output()
            .expect("failed to execute CMake");
        println!("status: {}", output.status);
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        assert!(output.status.success());

        let m = Command::new("cmake")
            .arg("--build")
            .arg(".")
            .arg("--config")
            .arg("Release")
            .output()
            .expect("failed to execute Make");
        println!("status: {}", m.status);
        std::io::stdout().write_all(&m.stdout).unwrap();
        std::io::stderr().write_all(&m.stderr).unwrap();
        assert!(m.status.success());
    } else {
        let c = Command::new("cmake")
            .arg(repo_dir.to_str().unwrap())
            .output()
            .expect("failed to execute CMake");
        println!("status: {}", c.status);
        std::io::stdout().write_all(&c.stdout).unwrap();
        std::io::stderr().write_all(&c.stderr).unwrap();
        assert!(c.status.success());

        let m = Command::new("make").output().expect("failed to execute Make");
        println!("status: {}", m.status);
        std::io::stdout().write_all(&m.stdout).unwrap();
        std::io::stderr().write_all(&m.stderr).unwrap();
        assert!(m.status.success());
    }

    env::set_current_dir(Path::new(&project_dir)).unwrap(); // change path back to main project

    if target.contains("windows") {
        let include = &build_dir.join("Release");
        println!("cargo:rustc-link-search=native={}", &include.to_str().unwrap());
        println!("cargo:rustc-link-lib=static=randomx");
    } else {
        println!("cargo:rustc-link-search=native={}", &build_dir.to_str().unwrap());
        println!("cargo:rustc-link-lib=static=randomx");
    } // link to RandomX

    if target.contains("apple") || target.contains("android") || target.contains("freebsd") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target.contains("windows") {
        // println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        unimplemented!();
    }
}
