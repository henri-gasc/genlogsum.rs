#![warn(missing_docs)]

//! Gentoo Log Summary
//!
//! A program that read emerge.log and output which packages are currently
//! being emerged, what time until they are done, and other things.

// // // // // // // // // // // // // // // // // // // // // // // //
//
// genlogsum: GENtoo LOG SUMmary, summarize log to show running emerge
// Copyright (C) 2024 Henri GASC
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
// // // // // // // // // // // // // // // // // // // // // // // //

use std::collections::HashMap;

use clap::Parser;
use genlogsum;

/// Read emerge log from file and put output in print.
///
/// * `file`: The file from which to create the record of past emerge.
/// * `config`: The configuration of the running program
/// * `fakeroot`: The root we will use to search and read mtimedb
/// * `print`: A string that will be modified to contains the status
/// * return an error if there was a problem when reading `file`
///
/// The reading of `[root]/var/cache/db/mtimedb` is done in this function, meaning if you used `--all` and told the program to read multiple files (using `--files`) from multiple root (with `--fakeroot`), your output will be polluted by it.
fn emerge_file(
    file: &str,
    config: &genlogsum::Arguments,
    fakeroot: &str,
    print: &mut String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut emerges_not_complete: HashMap<String, genlogsum::PackageInfo> = HashMap::new();
    let mut completed_atoms: HashMap<String, genlogsum::Atom> = HashMap::new();

    genlogsum::read_file(&file, &mut emerges_not_complete, &mut completed_atoms)?;
    genlogsum::set_last_time(&emerges_not_complete, &mut completed_atoms);

    if !config.all {
        for package in emerges_not_complete.values() {
            genlogsum::emerge_package(package, &completed_atoms, config, fakeroot, print);
        }
    } else {
        // Create next_emerge from data from mtimedb
        let list = genlogsum::read_mtimedb(fakeroot);
        for p in list {
            genlogsum::emerge_package_mtimedb(&p, &mut completed_atoms, print);
        }
    }

    return Ok(());
}

/// Go the root and read all files given by the config
///
/// * `fakeroot`: The folder we will use as root for the subsequent search for files
/// * `config`: The configuration of the running program
/// * `print`: A string that will be modified to contains the status
///
/// Something to note: if you are using `--all` for the program, the reading of `var/cache/db/mtimedb` is __NOT__ done in this function.
fn emerge_fakeroot(fakeroot: &str, config: &genlogsum::Arguments, print: &mut String) {
    for file in &config.files {
        let mut path = String::new();
        genlogsum::correct_path(fakeroot, file, &mut path);
        if let Err(e) = emerge_file(&path, config, fakeroot, print) {
            if !config.skip_file {
                eprintln!("Application error: {e} for {path}");
            }
        }
    }
}

/// The main function
///
/// This function only parse the arguments, call [`emerge_fakeroot`], and print the output.
fn main() {
    let args = &genlogsum::Arguments::parse();
    let mut print = String::new();

    for fakeroot in &args.fakeroots {
        emerge_fakeroot(fakeroot, args, &mut print);
    }

    if print.is_empty() {
        println!("Not currently emerging");
    } else {
        // There is a newline at the end of print
        print!("{print}");
    }
}
