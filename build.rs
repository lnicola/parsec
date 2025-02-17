// Copyright (c) 2019, Arm Limited, All Rights Reserved
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//          http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use cargo_toml::{Manifest, Value};
use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};

const CONFIG_TABLE_NAME: &str = "config";
const MBED_CRYPTO_VERSION_KEY: &str = "mbed-crypto-version";

const SETUP_MBED_SCRIPT_PATH: &str = "./setup_mbed_crypto.sh";
const BUILD_CONFIG_FILE_PATH: &str = "./build-conf.toml";

#[derive(Debug, Deserialize)]
struct Configuration {
    mbed_config: Option<MbedConfig>,
}

#[derive(Debug, Deserialize)]
struct MbedConfig {
    mbed_compiler: Option<String>,
    mbed_archiver: Option<String>,
    mbed_path: String,
}

fn get_configuration_string(plasma_config: &Value, key: &str) -> String {
    let config_value = get_value_from_table(plasma_config, key);
    match config_value {
        Value::String(string) => string.clone(),
        _ => panic!("Cargo.toml does not contain configuration key: {}", key),
    }
}

fn get_value_from_table<'a>(table: &'a Value, key: &str) -> &'a Value {
    match table {
        Value::Table(table) => table.get(key).expect(&format!(
            "Config table does not contain configuration key: {}",
            key
        )),
        _ => panic!("Value provided is not a TOML table"),
    }
}

// Get the Mbed Crypto version to branch on from Cargo.toml file. Use that and MbedConfig to pass
// parameters to the setup_mbed_crypto.sh script which clones and builds Mbed Crypto and create
// a static library.
fn setup_mbed_crypto(mbed_config: &MbedConfig) {
    let toml_path = std::path::Path::new("./Cargo.toml");
    if !toml_path.exists() {
        panic!("Could not find Cargo.toml.");
    }
    let manifest = Manifest::from_path(&toml_path).expect("Could not parse Cargo.toml.");
    let mbed_path = &mbed_config.mbed_path;

    let package = manifest
        .package
        .expect("Cargo.toml does not contain package information.");
    let metadata = package
        .metadata
        .expect("Cargo.toml does not contain package metadata.");
    let plasma_config = get_value_from_table(&metadata, CONFIG_TABLE_NAME);
    let mbed_version = get_configuration_string(&plasma_config, MBED_CRYPTO_VERSION_KEY);

    let mut run_script = ::std::process::Command::new(SETUP_MBED_SCRIPT_PATH);
    run_script.arg(mbed_version).arg(mbed_path);

    if let Some(mbed_compiler) = &mbed_config.mbed_compiler {
        run_script.arg(format!("CC={}", mbed_compiler));
    }
    if let Some(mbed_archiver) = &mbed_config.mbed_archiver {
        run_script.arg(format!("AR={}", mbed_archiver));
    }
    run_script
        .output()
        .expect("setup_mbed_crypto.sh script failed.");
}

fn generate_mbed_bindings(mbed_config: &MbedConfig) {
    let mbed_include_dir = mbed_config.mbed_path.clone() + "mbed-crypto/include";
    let project_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable was not found");

    println!(
        "cargo:rustc-link-search=native={}/target/mbed-crypto/library/",
        project_dir
    );
    println!("cargo:rustc-link-lib=static=mbedcrypto");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", mbed_include_dir))
        .rustfmt_bindings(true)
        .header(format!("{}/psa/crypto.h", mbed_include_dir))
        .generate_comments(false)
        .generate()
        .expect("Unable to generate bindings to mbed crypto");

    let out_path = PathBuf::from(project_dir);
    bindings
        .write_to_file(out_path.join("src/providers/mbed_provider/psa_crypto_binding.rs"))
        .expect(&format!("Couldn't write bindings to {:?}!", out_path));
}

// Get the compiler, the archiver and the location where to clone the Mbed Crypto repository.
fn parse_config_file() -> Configuration {
    let config_str = ::std::fs::read_to_string(Path::new(BUILD_CONFIG_FILE_PATH))
        .expect("Could not read configuration file.");
    toml::from_str(&config_str).expect("Could not parse build configuration file.")
}

fn main() {
    let config = parse_config_file();

    if cfg!(feature = "mbed") {
        let mbed_config = config.mbed_config.expect(&format!(
            "Could not find mbed_config table in the {} file.",
            BUILD_CONFIG_FILE_PATH
        ));
        setup_mbed_crypto(&mbed_config);
        generate_mbed_bindings(&mbed_config);
    }
}
