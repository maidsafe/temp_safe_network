#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! clap = "2.29.0"
//! colored = "1.6.0"
//! heck = "0.3.0"
//! toml = "0.4.5"
//! walkdir = "2.0.1"
//! zip = "0.2.6"
//! ```
extern crate clap;
extern crate colored;
extern crate heck;
extern crate toml;
extern crate walkdir;
extern crate zip;

use clap::{App, Arg};
use colored::*;
use heck::ShoutySnakeCase;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use walkdir::WalkDir;
use zip::ZipWriter;
use zip::write::FileOptions;

const CRATES: &[&str] = &["safe_app", "safe_authenticator"];

const ARCHS: &[Arch] = &[
    Arch {
        name: "linux-x86",
        target: "i686-unknown-linux-gnu",
        toolchain: "",
    },
    Arch {
        name: "linux-x64",
        target: "x86_64-unknown-linux-gnu",
        toolchain: "",
    },
    Arch {
        name: "osx-x86",
        target: "i686-apple-darwin",
        toolchain: "",
    },
    Arch {
        name: "osx-x64",
        target: "x86_64-apple-darwin",
        toolchain: "",
    },
    Arch {
        name: "win-x86",
        target: "i686-pc-windows-gnu",
        toolchain: "",
    },
    Arch {
        name: "win-x64",
        target: "x86_64-pc-windows-gnu",
        toolchain: "",
    },
    Arch {
        name: "android-armeabiv7a",
        target: "armv7-linux-androideabi",
        toolchain: "arm-linux-androideabi-",
    },
    Arch {
        name: "android-x86",
        target: "i686-linux-android",
        toolchain: "i686-linux-android-",
    },
    Arch {
        name: "ios-arm64",
        target: "aarch64-apple-ios",
        toolchain: "",
    },
    Arch {
        name: "ios-x86_64",
        target: "x86_64-apple-ios",
        toolchain: "",
    },
];


#[cfg(all(target_os = "linux", target_arch = "x86"))]
const HOST_ARCH_NAME: &str = "linux-x86";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const HOST_ARCH_NAME: &str = "linux-x64";
#[cfg(all(target_os = "macos", target_arch = "x86"))]
const HOST_ARCH_NAME: &str = "osx-x86";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const HOST_ARCH_NAME: &str = "osx-x64";
#[cfg(all(target_os = "windows", target_arch = "x86"))]
const HOST_ARCH_NAME: &str = "win-x86";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const HOST_ARCH_NAME: &str = "win-x64";

const BINDINGS_LANGS: &[&str] = &["csharp"];

const COMMIT_HASH_LEN: usize = 7;

fn main() {
    let arch_names: Vec<_> = ARCHS.into_iter().map(|args| args.name).collect();

    // Parse command line arguments.
    let matches = App::new("safe_client_libs packaging tool")
        .arg(
            Arg::with_name("NAME")
                .short("n")
                .long("name")
                .takes_value(true)
                .possible_values(CRATES)
                .required(true)
                .help("Name of the crate to package"),
        )
        .arg(Arg::with_name("COMMIT").short("c").long("commit").help(
            "Uses commit hash instead of version string in the package name",
        ))
        .arg(
            Arg::with_name("ARCH")
                .short("a")
                .long("arch")
                .takes_value(true)
                .possible_values(&arch_names)
                .help("Target platform and architecture"),
        )
        .arg(Arg::with_name("LIB").short("l").long("lib").help(
            "Generates library package",
        ))
        .arg(
            Arg::with_name("BINDINGS")
                .short("b")
                .long("bindings")
                .help("Generates bindings package"),
        )
        .arg(Arg::with_name("MOCK").short("m").long("mock").help(
            "Generates mock version of the library",
        ))
        .arg(
            Arg::with_name("TOOLCHAIN")
                .short("t")
                .long("toolchain")
                .takes_value(true)
                .help("Path to the toolchain (for cross-compilation)"),
        )
        .get_matches();

    let krate = matches.value_of("NAME").unwrap();
    let version_string = get_version_string(krate, matches.is_present("COMMIT"));
    let arch = matches.value_of("ARCH").and_then(|name| {
        ARCHS.into_iter().find(|arch| arch.name == name)
    });

    let bindings = matches.is_present("BINDINGS");
    let lib = matches.is_present("LIB");
    let mock = matches.is_present("MOCK");

    let toolchain_path = matches.value_of("TOOLCHAIN");
    let toolchain_prefix = get_toolchain_prefix(toolchain_path, arch);
    let strip_bin = format!("{}{}", toolchain_prefix, "strip");

    validate_env(arch);

    // Gather features.
    let mut features = vec![];
    if mock {
        features.push("use-mock-routing");
        features.push("testing");
    }
    if matches.is_present("BINDINGS") {
        features.push("bindings");
    }

    // Run the build.
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(format!("{}/Cargo.toml", krate));

    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }

    let target = arch.map(|arch| arch.target);
    if let Some(target) = target {
        command.arg("--target").arg(target);
    }

    if !command.status().unwrap().success() {
        return;
    }

    let file_options = FileOptions::default();

    // Create library archive.
    if lib {
        let libs = find_libs(krate, target);
        for path in &libs {
            strip_lib(&strip_bin, &path);
        }

        let archive_name = {
            let mock = if mock { "-mock" } else { "" };
            let arch_name = arch.map(|arch| arch.name).unwrap_or(HOST_ARCH_NAME);
            format!("{}{}-{}-{}.zip", krate, mock, version_string, arch_name)
        };

        let file = File::create(archive_name).unwrap();
        let mut archive = ZipWriter::new(file);

        for path in libs {
            archive
                .start_file(path.file_name().unwrap().to_string_lossy(), file_options)
                .unwrap();

            let mut file = File::open(path).unwrap();
            io::copy(&mut file, &mut archive).unwrap();
        }
    }

    // Create bindings archive.
    if bindings {
        let archive_name = format!("{}-bindings-{}.zip", krate, version_string);

        let file = File::create(archive_name).unwrap();
        let mut archive = ZipWriter::new(file);

        for lang in BINDINGS_LANGS {
            let source_prefix = Path::new("bindings").join(lang).join(krate);
            let target_prefix = Path::new(lang);

            for entry in WalkDir::new(&source_prefix) {
                let entry = entry.unwrap();
                let target_path =
                    target_prefix.join(entry.path().strip_prefix(&source_prefix).unwrap());
                let target_path = target_path.to_string_lossy();

                if entry.file_type().is_dir() {
                    archive.add_directory(target_path, file_options).unwrap();
                } else {
                    archive.start_file(target_path, file_options).unwrap();

                    let mut file = File::open(entry.path()).unwrap();
                    io::copy(&mut file, &mut archive).unwrap();
                }
            }
        }
    }
}

struct Arch {
    name: &'static str,
    target: &'static str,
    toolchain: &'static str,
}

fn get_version_string(krate: &str, commit: bool) -> String {
    if commit {
        // Get the current commit hash.
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .expect("failed to run git");

        str::from_utf8(&output.stdout).unwrap().trim()[0..COMMIT_HASH_LEN].to_string()
    } else {
        // Extract the version string from Cargo.toml
        use toml::Value;

        let mut file =
            File::open(Path::new(krate).join("Cargo.toml")).expect("failed to open Cargo.toml");
        let mut content = String::new();
        file.read_to_string(&mut content).expect(
            "failed to read Cargo.toml",
        );

        let toml = content.parse::<Value>().expect(
            "failed to parse Cargo.toml",
        );
        toml["package"]["version"]
            .as_str()
            .expect("failed to read package version from Cargo.toml")
            .to_string()
    }
}

fn get_toolchain_prefix(toolchain_path: Option<&str>, arch: Option<&Arch>) -> String {
    let mut result = PathBuf::new();

    if let Some(path) = toolchain_path {
        result.push(path);
        result.push("bin");
    }

    result.push(arch.map(|arch| arch.toolchain).unwrap_or(""));
    result.into_os_string().into_string().unwrap()
}

fn validate_env(arch: Option<&Arch>) {
    if let Some(arch) = arch {
        let name = format!("CARGO_TARGET_{}_LINKER", arch.target.to_shouty_snake_case());
        if let Ok(value) = env::var(&name) {
            if !Path::new(&value).exists() {
                println!(
                    "{}: the environment variable {} is set, but points to \
                     non-existing file {}. This might cause linker failures.",
                     "warning".yellow().bold(),
                    name.bold(),
                    value.bold(),
                );
            }
        } else {
            println!(
                "{}: the environment variable {} is not set. \
                 This might cause linker failure.",
                "warning".yellow().bold(),
                name.bold()
            );
        }
    }
}

fn find_libs(krate: &str, target: Option<&str>) -> Vec<PathBuf> {
    let mut prefix = PathBuf::from("target");
    if let Some(target) = target {
        prefix = prefix.join(target);
    }
    prefix = prefix.join("release");

    let mut result = Vec::with_capacity(1);

    // linux,osx - static
    let path = prefix.join(format!("lib{}.a", krate));
    if path.exists() {
        result.push(path);
    }

    // linux - dynamic
    let path = prefix.join(format!("lib{}.so", krate));
    if path.exists() {
        result.push(path);
    }

    // osx - dynamic
    let path = prefix.join(format!("lib{}.dylib", krate));
    if path.exists() {
        result.push(path);
    }

    // windows - dynamic
    let path = prefix.join(format!("{}.dll", krate));
    if path.exists() {
        result.push(path);
    }

    result
}

fn strip_lib(strip: &str, lib_path: &Path) {
    let mut command = Command::new(strip);

    // On OS X `strip` does not remove global symbols without this flag.
    if cfg!(target_os = "macos") {
        command.arg("-x");
    }

    command.arg(lib_path);

    if !command.status().expect("failed to run strip").success() {
        panic!("failed to strip {}", lib_path.display());
    }
}
