#![warn(missing_docs)]

//! This file should contains all functions used to parse the emerge log

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

use std::{collections::HashMap, error::Error, fs};

use crate::{
    package::{Atom, PackageInfo},
    useful::{get_size_cpn, LineType},
};

/// Build a [`PackageInfo`] struct with the information from `line`
///
/// Use some invariance in the lines to create a [PackageInfo] instance.
/// * `line`: The line that contains the information
/// * `start_index`: Where in `line` does the cpn start
/// * `found`: Where in `line` does the cpn end
/// * `time`: The time at which the line was written. Usually equal to the first 10 character of it
/// * `is_binary`: If the package we are building for is binary
/// * `end_symbol`: The symbol that mark the end of the full name of the package
fn build_package_info(
    line: &str,
    start_index: usize,
    found: usize,
    time: u32,
    is_binary: bool,
    end_symbol: char,
) -> Option<PackageInfo> {
    let cpn = &line[start_index..found];
    let space = start_index + &line[start_index..].find(end_symbol)?;
    let slash = cpn.find('/')?;

    let category = cpn[0..slash].to_string();
    let name = cpn[slash + 1..cpn.len()].to_string();
    let full_name = line[start_index..space].to_string();
    let num = line[line.find('(')? + 1..line.find(')')?].to_string();
    return Some(PackageInfo {
        category,
        name,
        full_name,
        time,
        is_binary,
        num,
    });
}

/// The default function. Used for the starting emerge lines.  
/// Use [`build_package_info`].
pub fn get_info(line: &str) -> Option<PackageInfo> {
    let time: u32 = line[0..line.find(':')?]
        .parse()
        .expect("Failed to convert the time to integer");

    // First character after the dash is a space
    let start_index = line.find(')').unwrap_or(line.len()) + 2;
    let found = start_index + get_size_cpn(&line[start_index..])?;

    return build_package_info(line, start_index, found, time, false, ' ');
}

/// As the name suggest, used for lines that have 3 equals (merging lines).  
/// Use [`build_package_info`].
fn get_info_3equal(line: &str, position: usize) -> Option<PackageInfo> {
    let mut pos = position;
    if pos == 0 {
        match line[24..].find(')') {
            Some(value) => pos = 24 + value,
            None => return None,
        }
    }

    let time: u32 = line[0..line.find(":")?]
        .parse()
        .expect("Failed to convert the time to integer");
    // The smallest word is "Merging" (len 10: ') ' + 7 + ' '
    let index_after_merge = pos + 10;

    // If we can not find the values ('(', ':', [cpn]), we have to return
    let start_index = index_after_merge + 1 + line[index_after_merge..].find('(')?;
    let end_pos = start_index + line[start_index..].find(':')?;
    let found = start_index + get_size_cpn(&line[start_index..end_pos])?;

    let is_binary = line[index_after_merge..].starts_with('B'); // ...) Merging Binary (xxx/yyy...)

    return build_package_info(line, start_index, found, time, is_binary, ':');
}

/// Complete an emerge.
///
/// * `complete_line`: The complete (merge) line
/// * `emerges_not_complete`: The HashMap that contains all emerges not yet completed.  
///   The package from `complete_line` is removed from it
/// * `completed_atoms`: The HashMap where we store the atoms. We add to it the package from `complete_line`
/// * `position`: A value to allow for slightly faster search time
fn complete_emerge(
    complete_line: &str,
    emerges_not_complete: &mut HashMap<String, PackageInfo>,
    completed_atoms: &mut HashMap<String, Atom>,
    position: usize,
) -> Option<bool> {
    let mut status = true;
    let package = get_info_3equal(complete_line, position)?;
    if package.is_binary {
        if emerges_not_complete.contains_key(&package.full_name) {
            emerges_not_complete.remove_entry(&package.full_name);
        }
        return Some(status);
    }

    if emerges_not_complete.contains_key(&package.full_name) {
        let p = emerges_not_complete.get(&package.full_name)?;

        // compare the package with the version
        if package.full_name == p.full_name {
            // If package is binary, then we dont want to count it
            // NOTE: May not be needed
            if p.is_binary {
                return Some(status);
            }

            // Time will never be less than 0
            let time: u32 = package.time - p.time;
            // let cpn = package.cpn();
            match completed_atoms.get_mut(&package.cpn()) {
                Some(atom) => atom.add(time),
                None => {
                    let a = Atom::new(package.cpn(), time, p.time);
                    completed_atoms.insert(package.cpn(), a);
                }
            }
        }
        emerges_not_complete.remove_entry(&package.full_name);
    } else {
        status = false;
    }

    return Some(status);
}

/// Return what is the type of `line` in the log. See [`LineType`].
fn select_line_type(line: &str) -> LineType {
    // The first 10 characters are used for the date. As such we can skip
    // them, as we have until the end of 2286 before we have to use 11
    // characters for the date, and we use 10 characters since 2001.

    // All lines of interest, are different in position 14
    let interesting = &line[13..18];

    if interesting.starts_with('>') && interesting.ends_with('e') {
        // Catch all '%d: >>> emerge %s'
        return LineType::START;
    } else if interesting.starts_with('=') && interesting.ends_with('(') {
        // We need to filter the merge messages
        if let Some(par) = line.find(')') {
            if let Some(letter) = line.as_bytes().get(par + 2) {
                if *letter == b'M' {
                    return LineType::MERGE;
                }
            }
        }
    } else if interesting.starts_with(':') && interesting.ends_with('c') {
        // End of a completed merge
        return LineType::END;
    } else if interesting.starts_with('*') && interesting.ends_with('t') {
        // Line of format '%d:  *** terminating.'
        return LineType::TERM;
    }
    return LineType::UNKNOW;
}

/// Select an action based on the line type
fn act_on_line(
    line: &str,
    emerges_not_complete: &mut HashMap<String, PackageInfo>,
    completed_atoms: &mut HashMap<String, Atom>,
) {
    // skip empty line or those starting with # (for testing purpose)
    if (line.len() == 0) || line.starts_with('#') {
        return;
    }

    match select_line_type(line) {
        LineType::START => {
            let pack = get_info(&line);
            let p;
            match pack {
                Some(info) => p = info,
                None => return,
            }
            emerges_not_complete.insert(p.full_name.clone(), p);
        }
        LineType::MERGE => {
            let par;
            match line[24..].find(')') {
                Some(value) => par = 24 + value,
                None => {
                    return;
                }
            }
            if line[par + 2..].starts_with('M') {
                // all merge => end of long time (for most emerge)
                let status = complete_emerge(line, emerges_not_complete, completed_atoms, par);

                if status.is_none() {
                    println!("Error when handling line {line}");
                }
            }
        }
        LineType::END => {
            let pack = get_info(&line);
            let p;
            match pack {
                Some(info) => p = info,
                None => return,
            }

            if let Some(m) = emerges_not_complete.get(&p.full_name) {
                // compare the packages with the version
                if (m.full_name == p.full_name) && !m.is_binary {
                    // Time will never be less than 0
                    let time = m.time - p.time;
                    match completed_atoms.get_mut(&m.cpn()) {
                        Some(atom) => atom.add(time),
                        None => {
                            let a = Atom::new(m.cpn(), time, p.time);
                            completed_atoms.insert(m.cpn(), a);
                        }
                    }
                }
                emerges_not_complete.remove_entry(&p.full_name);
            }
        }
        LineType::TERM => {
            emerges_not_complete.clear();
        }
        LineType::UNKNOW => {
            return;
        }
    }
}

/// Read the whole file given and update `emerges_not_complete` and `completed_atoms` as we go.
///
/// * `file`: The path as string to the file we want to read
/// * `emerges_not_complete`: The HashMap that contains all emerges not yet completed
/// * `completed_atoms`: The HashMap where we store the atoms.
pub fn read_file(
    file: &str,
    emerges_not_complete: &mut HashMap<String, PackageInfo>,
    completed_atoms: &mut HashMap<String, Atom>,
) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(file)?;

    for line in content.lines() {
        act_on_line(line, emerges_not_complete, completed_atoms);
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_info_without_colons() {
        let line = "146181";
        assert!(get_info(line).is_none());
    }

    #[test]
    #[should_panic]
    fn get_info_no_time() {
        let line = "aaaaa:";
        get_info(line);
    }

    #[test]
    #[should_panic]
    fn get_info_without_cpn() {
        let line = "146181: ";
        get_info(line);
    }

    #[test]
    fn get_info_with_cpn_none() {
        let line = "1: ) a/b-0";
        assert!(get_info(line).is_none());
    }

    #[test]
    fn get_info_with_cpn() {
        let line = "146181: (1 of 1) a/b-0 to /";
        let p = get_info(line).unwrap();
        assert_eq!(p.cpn(), "a/b".to_string());
        assert_eq!(p.full_name, "a/b-0".to_string());
        assert_eq!(p.time, 146181);
    }

    #[test]
    fn get_info_3equal_with_cpn() {
        let line = "1234567890:  === (1 of 1) Merging (app/testing-1.2.3::/var/db/repos/gentoo/app/testing/testing-1.2.3.ebuild)";
        let p = get_info_3equal(line, 0).unwrap();

        assert!(!p.is_binary);
        assert_eq!(p.time, 1234567890);
        assert_eq!(p.cpn(), "app/testing".to_string());
    }

    #[test]
    fn get_info_3equal_binary_with_cpn() {
        let line = "1234567890:  === (1 of 1) Merging Binary (app/testing-1.2.3::/)";
        let p = get_info_3equal(line, 24).unwrap();

        assert!(p.is_binary);
        assert_eq!(p.time, 1234567890);
        assert_eq!(p.cpn(), "app/testing".to_string());
    }

    #[test]
    #[should_panic]
    fn get_info_3equal_binary_panic() {
        let line = "1234567890:  === (1 of 1 Merging Binary (app/testing-1.2.3::/";
        get_info_3equal(line, 0).unwrap();
    }

    #[test]
    fn line_is_start() {
        let line = "1234567890:  >>> emerge (1 of 1) sys-devel/gcc-1.2.3 to /";
        assert!(std::matches!(select_line_type(line), LineType::START));
    }

    #[test]
    fn line_is_merge() {
        let line = "1234567890:  === (1 of 1) Merging something, does not matter";
        assert!(std::matches!(select_line_type(line), LineType::MERGE));
    }

    #[test]
    fn line_is_termination() {
        let line = "1234567890:  *** terminating.";
        assert!(std::matches!(select_line_type(line), LineType::TERM));
    }

    #[test]
    fn line_is_unknow() {
        let line = "1234567890:  >>> AUTOCLEAN: sec-policy/selinux-java:0";
        assert!(std::matches!(select_line_type(line), LineType::UNKNOW));
    }

    #[test]
    fn line_is_not_merging() {
        let line = "1234567890:  === (9 of 15) Cleaning (a/b-1.2.3::...";
        assert!(!std::matches!(select_line_type(line), LineType::MERGE));
        let line = "1234567890:  === (9 of 15) Post-Build Cleaning (a/b-1.2.3::...";
        assert!(!std::matches!(select_line_type(line), LineType::MERGE));
        let line = "1234567890:  === (9 of 15) Compiling/Packaging (a/b-1.2.3::...";
        assert!(!std::matches!(select_line_type(line), LineType::MERGE));
    }

    #[test]
    fn read_file_inexistent() {
        let file = "./tests/dont/exist";
        let mut emerges_not_complete: HashMap<String, PackageInfo> = HashMap::new();
        let mut completed_atoms: HashMap<String, Atom> = HashMap::new();
        assert!(read_file(file, &mut emerges_not_complete, &mut completed_atoms).is_err());
    }

    #[test]
    fn read_file_two_package_with_1binary() {
        let file = "./tests/emerge.log/two_with_1binary";
        let mut emerges_not_complete: HashMap<String, PackageInfo> = HashMap::new();
        let mut completed_atoms: HashMap<String, Atom> = HashMap::new();
        let result = read_file(file, &mut emerges_not_complete, &mut completed_atoms);

        assert!(result.is_ok());
        assert_eq!(emerges_not_complete.len(), 0);
        assert_eq!(completed_atoms.len(), 1); // Binary package are not added to it
    }
}
