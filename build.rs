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

use git2::{Cred, Oid, Repository};
use std::env;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    const RANDOMX_COMMIT: &str = "ac574e3743b00680445994cbe2c38ba0f52db70d";

    let out_dir = env::var("OUT_DIR").unwrap();
    let project_dir = Path::new(&out_dir);

    let repo_dir = project_dir.join("randomx");

    if !repo_dir.exists() {
        create_dir_all(&repo_dir.to_str().unwrap()).unwrap();

        // If we're inside CircleCI, use SSH (Circle requires this), otherwise good ol' https will do just fine
        let repo = match env::var("CIRCLECI") {
            Ok(v) if &v == "true" => build_using_ssh(&repo_dir),
            _ => build_using_https(&repo_dir),
        };

        let oid = Oid::from_str(RANDOMX_COMMIT).unwrap();
        let commit = repo.find_commit(oid).unwrap();

        let _branch = repo.branch(RANDOMX_COMMIT, &commit, false);

        let obj = repo
            .revparse_single(&("refs/heads/".to_owned() + RANDOMX_COMMIT))
            .unwrap();

        repo.checkout_tree(&obj, None).unwrap();

        repo.set_head(&("refs/heads/".to_owned() + RANDOMX_COMMIT))
            .unwrap();
    }

    env::set_current_dir(Path::new(&repo_dir)).unwrap(); //change current path to repo for dependency build
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        let c = Command::new("cmake")
            .arg("-G")
            .arg("Visual Studio 16 2019")
            .arg(".")
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
            .arg(".")
            .output()
            .expect("failed to execute CMake");
        println!("status: {}", c.status);
        std::io::stdout().write_all(&c.stdout).unwrap();
        std::io::stderr().write_all(&c.stderr).unwrap();
        assert!(c.status.success());
        let m = Command::new("make")
            .output()
            .expect("failed to execute Make");
        println!("status: {}", m.status);
        std::io::stdout().write_all(&m.stdout).unwrap();
        std::io::stderr().write_all(&m.stderr).unwrap();
        assert!(m.status.success());
    }

    env::set_current_dir(Path::new(&project_dir)).unwrap(); //change path back to main project

    if target.contains("windows") {
        let include = &repo_dir.join("Release");
        println!(
            "cargo:rustc-link-search=native={}",
            &include.to_str().unwrap()
        );
        println!("cargo:rustc-link-lib=static=randomx");
    } else {
        println!(
            "cargo:rustc-link-search=native={}",
            &repo_dir.to_str().unwrap()
        );
        println!("cargo:rustc-link-lib=randomx");
    } //link to RandomX

    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target.contains("windows") {
        //println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        unimplemented!();
    }
}

fn build_using_ssh(path: &Path) -> Repository {
    let url = "ssh://git@github.com/tevador/RandomX.git";
    // Build up auth credentials via fetch options:
    let mut cb = git2::RemoteCallbacks::new();
    cb.credentials(|_, _, _| {
        let credentials = Cred::ssh_key_from_agent("git").expect("Could not get SSH key");
        Ok(credentials)
    });
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);
    match builder.clone(url, &path) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to clone RandomX: {}", e),
    }
}

fn build_using_https(path: &Path) -> Repository {
    let url = "https://github.com/tevador/RandomX.git";
    match Repository::clone(url, &path) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to clone RandomX: {}", e),
    }
}
