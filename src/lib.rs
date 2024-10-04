use std::{collections::HashMap, fs};

use useful::Over;

pub use crate::json::read_mtimedb;
pub use crate::package::{Atom, PackageInfo};
pub use crate::parse_file::read_file;
pub use crate::useful::Arguments;

mod json;
mod package;
mod parse_file;
mod useful;

pub fn get_time_emerge(t: f32, over: Over) -> String {
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

pub fn set_last_time(
    emerges_not_complete: &HashMap<String, PackageInfo>,
    completed_atoms: &mut HashMap<String, Atom>,
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

fn ninja_read(p: &PackageInfo, output: &mut String) {
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

fn get_time_package(cpn: &str, completed_atoms: &HashMap<String, Atom>) -> (f32, Over) {
    let mut over = Over::NO;
    let time = match completed_atoms.get(cpn) {
        Some(atom) => atom.comp_avg(&mut over),
        None => -1.,
    };
    return (time, over);
}

fn get_time(r: &json::EmergeResume, completed_atoms: &HashMap<String, Atom>) -> (f32, Over) {
    // If package in waiting list is binary, add 2 minutes
    if r.binary {
        return (120.0, Over::NO);
    }
    // Otherwise, get the cpn from the name ...
    let size = useful::get_size_cpn(&r.name).unwrap_or(r.name.len());
    let cpn = &r.name.as_str()[..size];
    // ... and compute the time
    return get_time_package(cpn, completed_atoms);
}

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

fn status_package(
    emerge: &PackageInfo,
    completed_atoms: &HashMap<String, Atom>,
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

pub fn emerge_package(
    p: &PackageInfo,
    completed_atoms: &HashMap<String, Atom>,
    config: &Arguments,
    fakeroot: &str,
    print: &mut String,
) {
    let mut out = String::new();
    if config.show_root && (fakeroot != "/") {
        let name = std::path::Path::new(fakeroot).components().next_back();
        if let Some(val) = name {
            out.push_str(val.as_os_str().to_str().unwrap_or(""));
            out.push_str(": ");
        }
    }
    out.push_str(&status_package(p, completed_atoms, config, fakeroot).unwrap_or("".to_string()));

    print.push_str(&format!("{out}\n"));
}

pub fn emerge_package_mtimedb(
    emerge: &json::EmergeResume,
    completed_atoms: &mut HashMap<String, Atom>,
    print: &mut String,
) {
    let size = useful::get_size_cpn(&emerge.name).unwrap_or(emerge.name.len());
    let cpn = &emerge.name.as_str()[..size];
    if let Some(atom) = completed_atoms.get_mut(cpn) {
        atom.last_time = useful::current_time() as u32;
    }

    let (t, over) = get_time(&emerge, &completed_atoms);
    let mut output = String::from(&emerge.name);
    if t <= 0.0 {
        output.push_str(", Unknow");
    } else {
        output.push_str(&get_time_emerge(t, over));
    }

    print.push_str(&format!("{output}\n"));
}

#[cfg(test)]
mod tests {
    use super::*;

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
        map.insert(
            emerge.cpn(),
            Atom {
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
