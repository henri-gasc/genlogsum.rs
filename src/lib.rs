#![warn(missing_docs)]

//! Gentoo Log Summary
//!
//! Collection of function used in the binary (`gls` crate).

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

use std::{collections::HashMap, fs};

pub use crate::package::{Atom, PackageInfo};
pub use crate::parse_file::read_file;
pub use crate::useful::{correct_path, Arguments};

use crate::json::read_mtimedb;
use crate::useful::Over;

mod benchmark;
mod json;
mod package;
mod parse_file;
mod useful;

/// Return the time of an emerge as a string, with some more information
///
/// It uses [`Atom::convert_text`] to get the d h m representation of `t`.  
/// It prefaces this with a short text chosen accoring to `over`.
fn get_time_emerge(t: f64, over: Over) -> String {
    let mut output = String::new();
    let mut time = String::new();
    Atom::convert_text(t, &mut time);

    match over {
        Over::NO => output.push_str(", ETA:"),
        Over::AVG => output.push_str(", ETA (avg):"),
        Over::AVGWORST => output.push_str(", ETA (worst):"),
        Over::ALL => output.push_str(" is over by"),
    }

    return format!("{output} {}", &time[0..time.len() - 1]);
}

fn set_package_time(package: &PackageInfo, completed_atoms: &mut HashMap<String, Atom>) {
    match completed_atoms.get_mut(&package.cpn()) {
        Some(atom) => atom.last_time = package.time,
        None => (),
    }
}

/// This set the [Atom::last_time] in `completed_atoms` of all packages from `emerges_not_complete` (accoring to [PackageInfo::time]).
pub fn set_last_time(
    emerges_not_complete: &HashMap<String, PackageInfo>,
    completed_atoms: &mut HashMap<String, Atom>,
) {
    // Set the last emerge_time for all emerge not finished
    for emerge in emerges_not_complete {
        set_package_time(emerge.1, completed_atoms);
    }
}

/// Test the existence of the file `log_emerge`, and if found, return the last line
///
/// * `log_emerge`: The basis of the path for the log
/// * `time`: When the emerge could have been started
fn test_file(log_emerge: &str, time: u32) -> String {
    let mut path = log_emerge.to_string();
    let datetime = chrono::DateTime::from_timestamp(time.into(), 0).unwrap();
    let date = datetime.format("%Y%m%d-%H%M%S");

    path.push_str(&format!(":{date}.log"));

    return match std::fs::exists(&path) {
        Ok(true) => match fs::read_to_string(path) {
            Ok(content) => content.lines().last().unwrap_or("").to_string(),
            Err(_) => "".to_string(),
        },
        _ => "".to_string(),
    };
}

/// Read the advancement from the file in log/portage/build (that is why you need split-log in your FEATURE variable)
///
/// This function only read the last line (it uses [`test_file`]), so if the compiler show wome warnings, the progression will not appear.
fn ninja_read(p: &PackageInfo, output: &mut String) {
    let mut log_emerge = String::from("/var/log/portage/build/");
    log_emerge.push_str(&p.full_name);
    output.push_str(" ");

    // Test 3 files, as there may be slight delay between when the line was written in emerge.log, and when the file was created
    let mut line = test_file(&log_emerge, p.time + 1);
    if line == "" {
        line = test_file(&log_emerge, p.time);
        if line == "" {
            line = test_file(&log_emerge, p.time - 1);
        }
    }

    // Ninja show progress using '[x/y] cmd'
    if (line != "") && line.starts_with("[") {
        let first_char = line.as_bytes().get(1).unwrap_or(&b'a');
        let second = line.as_bytes().get(2).unwrap_or(&b'a');

        if useful::is_digit(first_char) || ((first_char == &b' ') && useful::is_digit(second)) {
            let end = line.find(']').unwrap_or(3) + 1;
            output.push_str(&line[0..end]);
        }
    }
}

/// Return the time taken by the package
///
/// Returns (-1, _) if the time is unknow (because never emerged before)
fn get_time_package(cpn: &str, completed_atoms: &HashMap<String, Atom>) -> (f64, Over) {
    let mut over = Over::NO;
    let time = match completed_atoms.get(cpn) {
        Some(atom) => atom.comp_avg(&mut over),
        None => -1.,
    };
    return (time, over);
}

/// Return the time the package would need to be installed
///
/// If we know the package is binary, then we get a shortcut
fn get_time(r: &json::EmergeResume, completed_atoms: &HashMap<String, Atom>) -> (f64, Over) {
    // If package in waiting list is binary, add 2 minutes
    if r.binary {
        return (120.0, Over::NO);
    }
    // Otherwise, get the cpn from the name ...
    let size = useful::get_size_cpn(&r.full_name).unwrap_or(r.full_name.len());
    let cpn = &r.full_name.as_str()[..size];
    // ... and compute the time
    return get_time_package(cpn, completed_atoms);
}

/// Read all the packages from mtimedb and add all their times.
///
/// If the time of one package in unknow, then the time for the sum is also unknow
///
/// * `fakeroot`: The folder from which we will try to access mtimedb
/// * `completed_atoms`: The HashMap of completed atoms
/// * `output`: Where the time will be placed after formatting
fn compile_resumelist(
    fakeroot: &str,
    completed_atoms: &HashMap<String, Atom>,
    output: &mut String,
) {
    let resume = read_mtimedb(fakeroot);
    let mut time = 0.0;
    for r in resume {
        let (t, _) = get_time(&r, completed_atoms);
        time += t;
        if t < 0.0 {
            // If the time is < 0, then we never encountered it and don't know
            output.push_str(", Total: Unknow");
            break;
        }
    }

    let mut out = String::from(", ");
    Atom::convert_text(time, &mut out);
    output.push_str(&out[..out.len() - 1]);
}

/// Put in output the time until the end.
/// Place 'Unknow' if time is less than zero
fn format_time(time: f64, over: Over, output: &mut String) {
    if time <= 0.0 {
        output.push_str(", Unknow");
    } else {
        output.push_str(&get_time_emerge(time, over));
    }
}

/// Get the status of a package
///
/// This return the formatted output of the package
///
/// * `emerge`: The package we want to know more about
/// * `completed_atoms`: The HashMap storing the completed atoms
/// * `config`: The configuration of the running program
/// * `fakeroot`: Will be passed to `compile_resumelist`, only used when `--full`
fn status_package(
    emerge: &PackageInfo,
    completed_atoms: &HashMap<String, Atom>,
    config: &Arguments,
    fakeroot: &str,
) -> Option<(String, f64)> {
    let time = useful::current_time() as u32;
    // If the emerge started a week ago, skip it
    if time - emerge.time > 7 * 24 * 60 * 60 {
        return None;
    }

    let mut output = String::new();
    if emerge.num != "" {
        output.push_str(&format!("{}, ", emerge.num));
    }
    output.push_str(&emerge.full_name);
    let (t, over) = get_time(
        &json::EmergeResume::create(emerge.is_binary, &emerge.cpn()),
        completed_atoms,
    );
    format_time(t, over, &mut output);

    if config.read_ninja {
        ninja_read(emerge, &mut output);
    }

    if config.format.full {
        compile_resumelist(fakeroot, completed_atoms, &mut output);
    }

    return Some((output, t));
}

/// Get the formatted output concerning a package
///
/// * `p`: The package we want more information on
/// * `completed_atoms`: The HashMap storing the completed atoms
/// * `config`: The configuration of the running program
/// * `fakeroot`: Where to search for mtimedb, and to change to name shown
/// * `print`: Where the formatted output will be put
///
/// # Examples
/// The kind of output will be like  
/// `1 of 2, sys-devel/gcc-13.3.1_p20240614, ETA: 3h 1m` for a classical output  
/// `gentoo: 51 of 51, media-gfx/krita-5.2.6 is over by a few seconds [225/3346]` for an output with --show-root --fakeroot /mnt/gentoo --read-ninja
fn emerge_package(
    p: &PackageInfo,
    completed_atoms: &HashMap<String, Atom>,
    config: &Arguments,
    fakeroot: &str,
    print: &mut String,
) -> f64 {
    let mut out = String::new();
    if config.show_root && (fakeroot != "/") {
        let name = std::path::Path::new(fakeroot).components().next_back();
        if let Some(val) = name {
            out.push_str(val.as_os_str().to_str().unwrap_or(""));
            out.push_str(": ");
        }
    }
    let (status, time) =
        status_package(p, completed_atoms, config, fakeroot).unwrap_or(("".to_string(), -1.0));
    out.push_str(&status);

    print.push_str(&format!("{out}\n"));
    return time;
}

/// The function you should use the get the emerge time for all packages in `emerges_not_complete` and in mtimedb if config allows you.
pub fn get_emerges(
    emerges_not_complete: &HashMap<String, PackageInfo>,
    completed_atoms: &mut HashMap<String, Atom>,
    config: &Arguments,
    fakeroot: &str,
    print: &mut String,
) {
    let mut total = 0.0;
    for package in emerges_not_complete.values() {
        let t = emerge_package(package, completed_atoms, config, fakeroot, print);
        total = useful::add_time(total, t);
    }

    if config.format.all {
        // Create next_emerge from data from mtimedb
        let list = read_mtimedb(fakeroot);
        for p in list {
            if let Some(_) = emerges_not_complete.get(&p.name) {
                continue;
            }
            let package = PackageInfo {
                category: p.category,
                name: p.name,
                full_name: p.full_name,
                time: useful::current_time() as u32,
                is_binary: p.binary,
                num: "".to_string(),
            };

            set_package_time(&package, completed_atoms);

            let t = emerge_package(&package, completed_atoms, config, fakeroot, print);
            total = useful::add_time(total, t);
        }

        let mut out = String::from("Total: ");
        if total < 0.0 {
            out.push_str("Unknow");
        } else {
            Atom::convert_text(total, &mut out);
            out = out[..out.len() - 1].to_string();
        }
        out.push('\n');

        print.push_str(&out);
    }
}

#[cfg(test)]
mod tests {
    use parse_file::read_file_test;

    use super::*;

    #[test]
    fn test_file_dont_exist() {
        let file = "/foo/bar";
        let time = 0;
        assert_eq!(test_file(file, time), "");
    }

    fn get_default_config() -> Arguments {
        return Arguments {
            files: vec!["./emerge.log".to_string()],
            fakeroots: vec!["/".to_string()],
            format: useful::Format {
                full: false,
                all: false,
            },
            read_ninja: false,
            show_root: false,
            skip_file: false,
        };
    }

    fn create_empty_hashmap() -> HashMap<String, Atom> {
        let m: HashMap<String, Atom> = HashMap::new();
        return m;
    }

    fn create_default_situation() -> (Arguments, HashMap<String, Atom>, PackageInfo) {
        let config = get_default_config();
        let emerge =
            parse_file::get_info("1234567890:  >>> emerge (1 of 1) app/testing-0.0.0 to /")
                .unwrap();
        let mut map = create_empty_hashmap();
        map.insert(emerge.cpn(), Atom::new(emerge.cpn(), 10, 0));
        return (config, map, emerge);
    }

    #[test]
    fn set_last_time_work() {
        let default = create_default_situation();
        let emerge = default.2;
        let cpn = emerge.cpn();
        let mut m = default.1;
        let mut map: HashMap<String, PackageInfo> = HashMap::new();
        map.insert(emerge.cpn(), emerge);
        map.insert("app/retesting".to_string(), create_default_situation().2); // The value will not changed after set_last_time
        assert_eq!(m.get(&cpn).unwrap().last_time, 0);
        set_last_time(&map, &mut m);
        assert_eq!(m.get(&cpn).unwrap().last_time, 1234567890);
    }

    #[test]
    fn status_package_over_time() {
        let default = create_default_situation();
        let mut emerge = default.2;
        emerge.time = 0;
        let map = default.1;
        let config = default.0;
        let status = status_package(&emerge, &map, &config, "/");
        assert!(status.is_none());
    }

    #[test]
    fn status_package_no_history() {
        let default = create_default_situation();
        let emerge = default.2;
        let mut map = default.1;
        map.clear();
        let config = default.0;
        let status = status_package(&emerge, &map, &config, "/");
        assert_eq!(status.unwrap().0, "1 of 1, app/testing-0.0.0, Unknow");
    }

    #[test]
    fn status_package_get_time() {
        let default = create_default_situation();
        let emerge = default.2;
        let map = default.1;
        let config = default.0;
        let status = status_package(&emerge, &map, &config, "/");
        assert_eq!(status.unwrap().0, "1 of 1, app/testing-0.0.0, ETA: 1m");
    }

    #[test]
    fn emerge_package_binary_running() {
        let (emerges_not_complete, completed_atoms) =
            read_file_test("./tests/emerge.log/binary_running");
        let config = get_default_config();
        let mut print = String::new();

        for package in emerges_not_complete.values() {
            emerge_package(package, &completed_atoms, &config, "/", &mut print);
        }

        assert_eq!(print, "1 of 1, category/package-1.2.3, ETA: 2m\n");
    }
}
