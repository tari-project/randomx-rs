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
extern crate git2;

use git2::Repository;
use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo_dir = format!("{}{}", project_dir, "/randomx/");

    if !Path::new(&repo_dir).exists() {
        let url = "https://github.com/tevador/RandomX.git";

        let _repo = match Repository::clone(url, "./randomx") {
            Ok(repo) => repo,
            Err(e) => panic!("Failed to clone RandomX: {}", e),
        };

        env::set_current_dir(Path::new(&repo_dir)).unwrap(); //change current path to repo for dependency build

        Command::new("cmake")
            .arg("-DARCH=native")
            .output()
            .expect("failed to execute CMake");

        Command::new("make")
            .output()
            .expect("failed to execute Make");

        env::set_current_dir(Path::new(&project_dir)).unwrap(); //change path back to main project
    };

    println!("cargo:rustc-link-search=native={}", &repo_dir);
    println!("cargo:rustc-link-lib=dylib=c++"); //link to c++
    println!("cargo:rustc-link-lib=randomx"); //link to RandomX
}
