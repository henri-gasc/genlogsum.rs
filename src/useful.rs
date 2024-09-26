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
}
