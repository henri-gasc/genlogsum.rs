use std::{collections::HashMap, error::Error, fs};

mod package;
mod useful;

enum LineType {
    START,
    MERGE,
    TERM,
    UNKNOW,
}

fn build_package_info(
    line: &str,
    start_index: usize,
    found: usize,
    time: u32,
    is_binary: bool,
    end_symbol: char,
) -> Option<package::PackageInfo> {
    let cpn = &line[start_index..found];
    let space = start_index + &line[start_index..].find(end_symbol)?;
    let slash = cpn.find('/')?;

    let category = cpn[0..slash].to_string();
    let name = cpn[slash + 1..cpn.len()].to_string();
    let full_name = line[start_index..space].to_string();
    return Some(package::PackageInfo {
        category,
        name,
        full_name,
        time,
        is_binary,
    });
}

fn get_info(line: &str) -> Option<package::PackageInfo> {
    let time: u32 = line[0..line.find(':')?]
        .parse()
        .expect("Failed to convert the time to integer");

    // First character after the dash is a space
    let start_index = line.find(')').unwrap_or(line.len()) + 2;
    let found = start_index + useful::get_size_cpn(&line[start_index..])?;

    return build_package_info(line, start_index, found, time, false, ' ');
}

fn get_info_3equal(line: &str, position: usize) -> Option<package::PackageInfo> {
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
    let found = start_index + useful::get_size_cpn(&line[start_index..end_pos])?;

    let is_binary = line[index_after_merge..].starts_with('B'); // ...) Merging Binary (xxx/yyy...)

    return build_package_info(line, start_index, found, time, is_binary, ':');
}

fn complete_emerge(
    complete_line: &str,
    emerges_not_complete: &mut HashMap<String, package::PackageInfo>,
    completed_atoms: &mut HashMap<String, package::Atom>,
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
                    let a = package::Atom::new(package.cpn(), time, p.time);
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
        // The real meat of the log is here
        return LineType::MERGE;
    } else if interesting.starts_with('*') && interesting.ends_with('t') {
        // Line of format '%d:  *** terminating.'
        return LineType::TERM;
    }
    return LineType::UNKNOW;
}

pub fn read_file() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("/var/log/emerge.log")?;
    let mut emerge_not_complete: HashMap<String, package::PackageInfo> = HashMap::new();
    let mut completed_atoms: HashMap<String, package::Atom> = HashMap::new();

    for line in content.lines() {
        // skip empty line or those starting with # (for testing purpose)
        if line.len() == 0 || line.starts_with('#') {
            continue;
        }

        match select_line_type(line) {
            LineType::START => {
                let pack = get_info(&line);
                let p;
                match pack {
                    Some(info) => p = info,
                    None => {
                        continue;
                    }
                }
                emerge_not_complete.insert(p.full_name.clone(), p);
            }
            LineType::MERGE => {
                let par;
                match line[24..].find(')') {
                    Some(value) => par = value,
                    None => {
                        continue;
                    }
                }
                if line[par + 2..].starts_with('M') {
                    // all merge => end of long time (for most emerge)
                    let status =
                        complete_emerge(line, &mut emerge_not_complete, &mut completed_atoms, par);

                    if status.is_none() {
                        println!("Error when handling line {line}");
                    }
                }
            }
            LineType::TERM => {
                emerge_not_complete.clear();
            }
            LineType::UNKNOW => {
                continue;
            }
        }
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
        let line = "146181: ) a/b-0 to /";
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
        let p = get_info_3equal(line, 0).unwrap();

        assert!(p.is_binary);
        assert_eq!(p.time, 1234567890);
        assert_eq!(p.cpn(), "app/testing".to_string());
    }

    #[test]
    fn line_is_start() {
        let line = "1234567890:  >>> emerge (1 of 1) sys-devel/gcc-1.2.3 to /";
        assert!(std::matches!(select_line_type(line), LineType::START));
    }

    #[test]
    fn line_is_merge() {
        let line = "1234567890:  === (1 of 1) Merging. does not matter";
        assert!(std::matches!(select_line_type(line), LineType::MERGE));
    }

    #[test]
    fn line_is_termination() {
        let line = "1234567890:  *** terminating.";
        assert!(std::matches!(select_line_type(line), LineType::TERM));
    }
}
