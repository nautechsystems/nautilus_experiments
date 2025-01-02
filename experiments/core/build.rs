// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2022 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

extern crate cbindgen;

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let crate_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var is not defined"),
    );

    // Generate C headers
    let config_c = cbindgen::Config::from_file("cbindgen.toml")
        .expect("Unable to find cbindgen.toml configuration file");

    cbindgen::generate_with_config(&crate_dir, config_c.clone())
        .expect("Unable to generate bindings")
        .write_to_file(crate_dir.join("../data/includes/core.h"));

    // Generate Cython definitions
    let config_cython = cbindgen::Config::from_file("cbindgen_cython.toml")
        .expect("Unable to find cbindgen.toml configuration file");

    let pxd_path = crate_dir.join("../data/rust/core.pxd");

    cbindgen::generate_with_config(&crate_dir, config_cython)
        .expect("Unable to generate bindings")
        .write_to_file(&pxd_path);

    // Post-process the .pxd file to add uint128_t definition
    let content = fs::read_to_string(&pxd_path).expect("Unable to read .pxd file");
    let lines: Vec<&str> = content.lines().collect();

    let mut output = String::new();
    let mut found_extern = false;

    for line in lines {
        output.push_str(line);
        output.push('\n');

        if line.trim().starts_with("cdef extern from") && !found_extern {
            output.push_str("    ctypedef unsigned long long uint128_t\n");
            output.push_str("    ctypedef long long int128_t\n");
            found_extern = true;
        }
    }

    // Write the modified content back to the file
    let mut file = fs::File::create(&pxd_path).expect("Unable to open .pxd file for writing");
    file.write_all(output.as_bytes())
        .expect("Unable to write to .pxd file");
}
