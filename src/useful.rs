#![warn(missing_docs)]

//! Store useful structures and functions

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

use clap::Parser;

/// Enum type for the time of an emerge and its relashionship with the previous times of the package
pub enum Over {
    /// The time of the emerge is under the average
    NO,
    /// The time is over the filtered (without worst and best) average, but under the true (with worst and best) average
    AVG,
    /// the time is over the true average, but under the worst time
    AVGWORST,
    /// The time is over everything, This will be the new worst time when the emerge is done
    ALL,
}

/// Enum type for what the line in emerge.log is
pub enum LineType {
    /// If the line if the starting point of an emerge
    START,
    /// If the line corresponds to the merge of an emerge
    MERGE,
    /// If the line signal termination
    TERM,
    /// If the line is not from the previous 3 types
    UNKNOW,
}

#[derive(Parser, Default, Debug)]
#[command(
    author = "Henri GASC",
    version,
    about = "Parse Gentoo emerge log files and output the status"
)]
/// Structures to store the configuration and arguments given from the command line
pub struct Arguments {
    #[arg(short, long, default_value = "/var/log/emerge.log", num_args(1..))]
    /// Add a file to be read.
    pub files: Vec<String>,

    #[arg(long, default_value = "/", num_args(1..), verbatim_doc_comment, long_help="Should be a folder where you can chroot in as we will use the paths [root]/[file] and [root]/var/cache/edb/mtimedb.\nThis option is chained with <FILES>, meaning \"-f foo.log foo/bar.log --fakeroots /foo / bar\" will search for:\n\t/foo/foo.log, /foo/foo/bar.log, /bar/foo.log, /bar/foo/bar.log")]
    /// Select a folder to act as root.
    pub fakeroots: Vec<String>,

    #[arg(long)]
    /// Print the total time until the end of the emerge command.
    pub full: bool,

    #[arg(long)]
    /// Print the time needed for all packages in mtimedb.
    pub all: bool,

    #[arg(long)]
    /// Read the completion rate from the log.
    /// Your portage need split-log in FEATURES.
    pub read_ninja: bool,

    #[arg(long)]
    /// Print the name of root we used.
    pub show_root: bool,

    #[arg(long)]
    /// If an error was found while reading a file, do not report the error.
    pub skip_file: bool,
}

/// Return the current time (the number of seconds since EPOCH)
pub fn current_time() -> u64 {
    #[cfg(not(test))]
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time was warped. Fix it !")
        .as_secs();

    #[cfg(test)]
    return 1234567890;
}

/// Test wether or not `c` is a digit
pub fn is_digit(c: &u8) -> bool {
    return (&b'0' <= c) && (c <= &b'9');
}

/// Compute the size of category/name from category/name-version
///
/// Grow the window until the character after a `-` is a digit.  
/// Theorically, could give false positive, in practice, I don't care.
pub fn get_size_cpn(cpnpv: &str) -> Option<usize> {
    let mut n = 0;
    let mut found: bool = false;
    while !found {
        match cpnpv[n..].find("-") {
            Some(value) => n += value + 1,
            None => {
                n = cpnpv.len() + 1;
                break;
            }
        }
        found = is_digit(cpnpv.as_bytes().get(n)?);
    }

    return Some(n - 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_tests() {
        assert!(!is_digit(&b'a'));
        assert!(!is_digit(&b'A'));
        assert!(is_digit(&b'0'));
        assert!(is_digit(&b'9'));
        assert!(is_digit(&b'6'));
    }
    #[test]
    fn test_size_cpn_simple() {
        let a = "hello";
        let s_a = get_size_cpn(a).unwrap();
        assert_eq!(s_a, a.len());
    }

    #[test]
    fn test_size_cpn_dash() {
        let a = "sys-devel/gcc";
        let s_a = get_size_cpn(a).unwrap();
        assert_eq!(s_a, a.len());
    }

    #[test]
    fn test_size_cpn_version() {
        let gcc = "sys-devel/gcc";
        let a = "sys-devel/gcc-12.4.0";

        let s_a = get_size_cpn(a).unwrap();

        assert_eq!(s_a, gcc.len());
    }

    #[test]
    fn test_size_cpn_hard() {
        let a = "dev-python/PyQt6-6.7.1-r1";
        let s_a = get_size_cpn(a).unwrap();
        assert_eq!(s_a, 16);
    }

    #[test]
    fn correct_time() {
        assert_eq!(current_time(), 1234567890);
    }
}
