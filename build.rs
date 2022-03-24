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

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let project_dir = Path::new(&out_dir);
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let repo_dir = PathBuf::from(env::var("RANDOMX_DIR").unwrap_or_else(|_| format!("{}/RandomX", &cargo_dir)));
    let build_dir = &project_dir.join("randomx_build");

    env::set_current_dir(Path::new(&repo_dir)).unwrap(); // change current path to repo for dependency build
    match fs::create_dir_all(&build_dir) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => (),
            _ => panic!("{}", e),
        },
    }
    env::set_current_dir(build_dir).unwrap();
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
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

    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target.contains("windows") {
        // println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        unimplemented!();
    }
}
