use std::{collections::HashMap, error::Error, fs};

mod json;
pub mod package;
use json::read_mtimedb;

pub use crate::useful::Arguments;
use crate::useful::{LineType, Over};
mod useful;

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
    let num = line[line.find('(')? + 1..line.find(')')?].to_string();
    return Some(package::PackageInfo {
        category,
        name,
        full_name,
        time,
        is_binary,
        num,
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

fn get_time_emerge(t: f32, over: Over) -> String {
    let mut output = String::new();
    let mut time = String::new();
    package::Atom::convert_text(t, &mut time);

    match over {
        Over::NO => output.push_str(", ETA:"),
        Over::AVG => output.push_str(", ETA (avg):"),
        Over::AVGWORST => output.push_str(", ETA (worst):"),
        Over::ALL => output.push_str(" is over by"),
    }

    return format!("{output} {}", &time[0..time.len() - 1]);
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

pub fn read_file(
    file: &str,
    emerges_not_complete: &mut HashMap<String, package::PackageInfo>,
    completed_atoms: &mut HashMap<String, package::Atom>,
) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(file)?;

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
                emerges_not_complete.insert(p.full_name.clone(), p);
            }
            LineType::MERGE => {
                let par;
                match line[24..].find(')') {
                    Some(value) => par = 24 + value,
                    None => {
                        continue;
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
            LineType::TERM => {
                emerges_not_complete.clear();
            }
            LineType::UNKNOW => {
                continue;
            }
        }
    }

    return Ok(());
}

pub fn set_last_time(
    emerges_not_complete: &HashMap<String, package::PackageInfo>,
    completed_atoms: &mut HashMap<String, package::Atom>,
) {
    // Set the last emerge_time for all emerge not finished
    for emerge in emerges_not_complete {
        match completed_atoms.get_mut(&emerge.1.cpn()) {
            Some(atom) => atom.last_time = emerge.1.time,
            None => continue,
        }
    }
}

fn test_file(log_emerge: &str, time: u32) -> String {
    let mut path = log_emerge.to_string();
    let datetime = chrono::DateTime::from_timestamp(time.into(), 0).unwrap();
    let date = datetime.format("%Y%m%d-%H%M%S").to_string();

    path.push_str(&format!(":{date}.log"));

    return match std::fs::exists(&path) {
        Ok(true) => match fs::read_to_string(path) {
            Ok(content) => content.lines().last().unwrap_or("").to_string(),
            Err(_) => "".to_string(),
        },
        _ => "".to_string(),
    };
}

pub fn ninja_read(p: &package::PackageInfo, output: &mut String) {
    let mut log_emerge = String::from("/var/log/portage/build/");
    log_emerge.push_str(&p.full_name);
    output.push_str(" ");

    let mut line = test_file(&log_emerge, p.time + 1);
    if line == "" {
        line = test_file(&log_emerge, p.time);
        if line == "" {
            line = test_file(&log_emerge, p.time - 1);
        }
    }

    if (line != "") && line.starts_with('[') {
        let mut start: i32 = -1;
        if useful::is_digit(line.as_bytes().get(1).unwrap_or(&b'a')) {
            start = 1;
        } else if (line.as_bytes().get(1).unwrap_or(&b'a') == &b' ')
            && useful::is_digit(line.as_bytes().get(2).unwrap_or(&b'a'))
        {
            start = 2;
        }

        if start >= 1 {
            let end = line.find(']').unwrap_or(3);
            output.push_str(&line[0..end + 1]);
        }
    }
}

fn get_time_package(cpn: &str, completed_atoms: &HashMap<String, package::Atom>) -> (f32, Over) {
    let mut over = Over::NO;
    let time = match completed_atoms.get(cpn) {
        Some(atom) => atom.comp_avg(&mut over),
        None => -1.,
    };
    return (time, over);
}

fn compile_resumelist(
    fakeroot: &str,
    completed_atoms: &HashMap<String, package::Atom>,
    output: &mut String,
) {
    let resume = read_mtimedb(fakeroot);
    let mut time = 0.0;
    for r in resume {
        // If package in waiting list is binary, add 2 minutes
        if r.binary {
            time += 120.0;
        } else {
            // Otherwise, get the cpn from the name ...
            let size = useful::get_size_cpn(&r.name).unwrap_or(r.name.len());
            let cpn = &r.name.as_str()[..size];
            // ... and compute the time
            let (t, _) = get_time_package(cpn, completed_atoms);
            if t < 0.0 {
                // If the time is < 0, then we never encountered it and don't know
                output.push_str(", Total: Unknow");
                break;
            }
            time += t;
        }
    }

    let mut out = String::from(", ");
    package::Atom::convert_text(time, &mut out);
    output.push_str(&out[..out.len() - 1]);
}

pub fn status_package(
    emerge: &package::PackageInfo,
    completed_atoms: &HashMap<String, package::Atom>,
    config: &Arguments,
    fakeroot: &str,
) -> Option<String> {
    let time = useful::current_time() as u32;
    // If the emerge started a week ago, skip it
    if time - emerge.time > 7 * 24 * 60 * 60 {
        return None;
    }

    let mut output = format!("{}, {}", emerge.num, emerge.full_name);
    let (t, over) = get_time_package(&emerge.cpn(), completed_atoms);
    if t <= 0.0 {
        output.push_str(", Unknow");
    } else {
        output.push_str(&get_time_emerge(t, over));
    }

    if config.read_ninja {
        ninja_read(emerge, &mut output);
    }

    if config.full {
        compile_resumelist(fakeroot, completed_atoms, &mut output);
    }

    return Some(output);
}

pub fn correct_path(root: &str, file: &str, path: &mut String) {
    if !file.starts_with('.') {
        path.push_str(root);
        if !root.ends_with('/') {
            path.push_str("/");
        }
    }

    let mut start_file = 0;
    if file.starts_with('/') {
        start_file = 1;
    }
    path.push_str(&file[start_file..]);
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
    fn correct_path_classical() {
        let root = "/";
        let file = "/var/log/emerge.log";
        let mut path = String::new();
        let expected = "/var/log/emerge.log";

        correct_path(root, file, &mut path);
        assert_eq!(path, expected);
    }

    #[test]
    fn correct_path_chroot() {
        let root = "/mnt/gentoo";
        let file = "var/log/emerge.log";
        let mut path = String::new();
        let expected = "/mnt/gentoo/var/log/emerge.log";

        correct_path(root, file, &mut path);
        assert_eq!(path, expected);
    }

    #[test]
    fn correct_path_stupid() {
        let root = "/";
        let file = "./emerge.log";
        let mut path = String::new();
        let expected = "./emerge.log";

        correct_path(root, file, &mut path);
        assert_eq!(path, expected);
    }

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
            full: false,
            all: false,
            read_ninja: false,
            show_root: false,
            skip_file: false,
        };
    }

    fn create_empty_hashmap() -> HashMap<String, package::Atom> {
        let m: HashMap<String, package::Atom> = HashMap::new();
        return m;
    }

    fn create_default_situation() -> (
        Arguments,
        HashMap<String, package::Atom>,
        package::PackageInfo,
    ) {
        let config = get_default_config();
        let emerge = get_info("1234567890:  >>> emerge (1 of 1) app/testing-0.0.0 to /").unwrap();
        let mut map = create_empty_hashmap();
        map.insert(
            emerge.cpn(),
            package::Atom {
                cpn: emerge.cpn(),
                num_emerge: 1,
                total_time: 10,
                best_time: 10,
                worst_time: 10,
                last_time: 0,
            },
        );
        return (config, map, emerge);
    }

    #[test]
    fn set_last_time_work() {
        let default = create_default_situation();
        let emerge = default.2;
        let cpn = emerge.cpn();
        let mut m = default.1;
        let mut map: HashMap<String, package::PackageInfo> = HashMap::new();
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
        assert_eq!(status.unwrap(), "1 of 1, app/testing-0.0.0, Unknow");
    }

    #[test]
    fn status_package_get_time() {
        let default = create_default_situation();
        let emerge = default.2;
        let map = default.1;
        let config = default.0;
        let status = status_package(&emerge, &map, &config, "/");
        assert_eq!(status.unwrap(), "1 of 1, app/testing-0.0.0, ETA: 1m");
    }
}
