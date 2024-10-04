use clap::Parser;

pub enum Over {
    NO,       // Not over anything
    AVG,      // Over average, under average with worst
    AVGWORST, // Over average with worst, under worst
    ALL,      // Over worst
}

pub enum LineType {
    START,
    MERGE,
    TERM,
    UNKNOW,
}

#[derive(Parser, Default, Debug)]
#[command(author = "Henri GASC", version, about)]
/// Parse Gentoo emerge log files and output the status
pub struct Arguments {
    #[arg(short, long, default_value = "/var/log/emerge.log", num_args(1..))]
    /// Add a file to be read.
    pub files: Vec<String>,

    #[arg(long, default_value = "/", num_args(1..), verbatim_doc_comment, long_help="Should be a folder where you can chroot in as we will use the paths [root]/[file] and [root]/var/cache/edb/mtimedb.\nThis option is chained with <FILES>, meaning \"-f foo.log foo/bar.log --fakeroots /foo / bar\" will search for:\n\t/foo/foo.log, /foo/foo/bar.log, /bar/foo.log, /bar/foo/bar.log")]
    /// Select a folder to act as root.
    pub fakeroots: Vec<String>,

    #[arg(long)]
    /// Read the completion rate from the log.
    /// Your portage need split-log in FEATURES.
    pub read_ninja: bool,

    #[arg(long)]
    /// If an error was found while reading a file, do not report the error.
    pub skip_file: bool,
}

#[cfg(not(test))]
pub fn current_time() -> u64 {
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time was warped. Fix it !")
        .as_secs();
}

#[cfg(test)]
pub fn current_time() -> u64 {
    return 1234567890;
}

pub fn is_digit(c: &u8) -> bool {
    return (&b'0' <= c) && (c <= &b'9');
}

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
