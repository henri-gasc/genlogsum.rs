use crate::useful::{current_time, Over};

/// A structure to store the data until we find a line that allows us to either discard it, or add it to the list of Atoms
pub struct PackageInfo {
    pub category: String,  // the category of the package
    pub name: String,      // the package name
    pub full_name: String, // the full name of the package (including version, revision, status)
    pub time: u32,         // The complete line in the file
    pub is_binary: bool,   // is it a binary emerge
    pub num: String,       // The number (x of y)
}

impl PackageInfo {
    /// Return the category/package_name representation of the package
    pub fn cpn(&self) -> String {
        return format!("{}/{}", self.category, self.name);
    }
}

pub struct Atom {
    pub cpn: String,     // the category/package-name representation
    pub num_emerge: u32, // the number of time the package was emerged
    pub total_time: u32, // the total emerge time
    pub best_time: u32,  // the shortest time it took to emerge this package
    pub worst_time: u32, // the longest time it took to emerge this package
    pub last_time: u32,  // the last time an emerge was started (avoid using PackageInfo)
}

impl Atom {
    /// Create a new instance of Atom with already a time
    pub fn new(cpn: String, time: u32, last_time: u32) -> Self {
        return Self {
            cpn,
            num_emerge: 1,
            total_time: time,
            best_time: time,
            worst_time: time,
            last_time,
        };
    }

    /// Add an emerge time to the package
    pub fn add(&mut self, time: u32) {
        self.num_emerge += 1;
        self.total_time += time;
        self.worst_time = std::cmp::max(self.worst_time, time);
        self.best_time = std::cmp::min(self.best_time, time);
    }

    /// Compute the average time with filter
    fn filter_time(&self) -> f32 {
        let mut t = self.total_time;
        let mut n = self.num_emerge;
        if n > 2 {
            t -= self.best_time;
            t -= self.worst_time;
            n -= 2;
        }
        return (t / n) as f32;
    }

    /// Return the average time for an emerge
    fn time_avg(&self) -> f32 {
        return (self.total_time / self.num_emerge) as f32;
    }

    /// Compute the average time for the emerge, along with the filters needed
    fn comp_avg(&self, over: &mut Over) -> f32 {
        // time between the start of the emerge and now
        let now = current_time() as u32;

        let mut diff: f32 = 0.;
        if self.last_time != 0 {
            diff = (now - self.last_time) as f32;
        }

        // Compute the time diff between the average and now
        let mut avg = self.filter_time() - diff;
        if avg < 0. {
            *over = Over::AVG;
            avg = self.time_avg() - diff;
            if avg < 0. {
                *over = Over::AVGWORST;
                avg = self.worst_time as f32 - diff;
                if avg < 0. {
                    *over = Over::ALL;
                    // Give the time diff with the worst emerge
                    avg = -avg;
                }
            }
        }

        // Add 25% of the time, only if are using the average filtered or the complete average
        if matches!(over, Over::NO) || matches!(over, Over::AVG) {
            // Add 25% to the time, and prepare for the rounding
            avg = avg * 1.25 + 60.;
        }

        return avg;
    }

    /// Format time to be on the format d h m, or with other special text
    fn convert_text(&self, time: f32, out: &mut String) {
        let d = (time / (60. * 60. * 24.)) as u32;
        let h = ((time / (60. * 60.)) % 24.) as u32;
        let m = (((time / 60.) % (60. * 24.)) % 60.) as u32;
        if (d == 0) && (h == 0) && (m == 0) {
            *out = "a few seconds".to_string();
        }

        if d != 0 {
            out.push_str(&format!("{}d ", d.to_string()));
        }
        if h != 0 {
            out.push_str(&format!("{}h ", h.to_string()));
        }
        if m != 0 {
            out.push_str(&format!("{}m ", m.to_string()));
        }
    }

    /// Format the time of Atom to "xd yh zm" format with special format, should avg - diff yield a negative result
    pub fn return_time(&self, time: &mut String) -> Over {
        let mut over = Over::NO;
        let avg = self.comp_avg(&mut over);
        self.convert_text(avg, time);
        return over;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_atom(time: u32) -> Atom {
        return Atom::new("cpn".to_string(), time, 0);
    }

    #[test]
    fn package_info_cpn() {
        let p = PackageInfo {
            category: "a".to_string(),
            name: "b".to_string(),
            full_name: "a/b-0.0.1".to_string(),
            time: 1,
            is_binary: false,
            num: "".to_string(),
        };

        assert_eq!(p.cpn(), "a/b");
    }

    #[test]
    fn atom_new() {
        let time = 15;
        let p = setup_atom(15);

        assert_eq!(p.cpn, "cpn".to_string());
        assert_eq!(p.num_emerge, 1);
        assert_eq!(p.best_time, time);
        assert_eq!(p.worst_time, time);
        assert_eq!(p.total_time, time);
    }

    #[test]
    fn atom_add() {
        let time = 15;
        let mut p = setup_atom(0);
        p.add(time);

        assert_eq!(p.cpn, "cpn".to_string());
        assert_eq!(p.num_emerge, 2);
        assert_eq!(p.best_time, 0);
        assert_eq!(p.worst_time, time);
        assert_eq!(p.total_time, time);

        p.add(time);
        assert_eq!(p.total_time, 2 * time);
    }

    #[test]
    fn atom_filter_time_2() {
        let mut atom = setup_atom(10);
        atom.add(30);
        assert_eq!(atom.total_time, 40);
        assert_eq!(atom.filter_time(), 20 as f32);
        assert_eq!(atom.time_avg(), 20 as f32);
    }

    #[test]
    fn atom_filter_time_4_same() {
        let mut atom = setup_atom(10);
        atom.add(10);
        atom.add(10);
        atom.add(10);

        assert_eq!(atom.total_time, 40);
        assert_eq!(atom.filter_time(), 10 as f32);
        assert_eq!(atom.time_avg(), 10 as f32);
    }

    #[test]
    fn atom_filter_time_4_diff() {
        let mut atom = setup_atom(10);
        atom.add(20);
        atom.add(30);
        atom.add(40);

        assert_eq!(atom.total_time, 100);
        assert_eq!(atom.filter_time(), (50 / 2) as f32);
        assert_eq!(atom.time_avg(), (100 / 4) as f32);
    }

    #[test]
    fn atom_convert_text_none() {
        let mut out = String::new();
        let atom = setup_atom(0);
        atom.convert_text(0., &mut out);
        assert_eq!(out, "a few seconds".to_string());
    }

    #[test]
    fn atom_convert_text_seconds() {
        let mut out = String::new();
        let atom = setup_atom(0);
        atom.convert_text(32., &mut out);
        assert_eq!(out, "a few seconds".to_string());
    }

    #[test]
    fn atom_convert_text_minutes() {
        let mut out = String::new();
        let atom = setup_atom(0);
        atom.convert_text(29. * 60. + 27., &mut out);
        assert_eq!(out, "29m ".to_string());
    }

    #[test]
    fn atom_convert_text_hours() {
        let mut out = String::new();
        let atom = setup_atom(0);
        atom.convert_text((71 * 60 + 61) as f32, &mut out);
        assert_eq!(out, "1h 12m ".to_string());
    }

    #[test]
    fn atom_convert_text_days() {
        let mut out = String::new();
        let atom = setup_atom(0);
        atom.convert_text((91 * 24 * 60 * 60 + 9 * 60 * 60 + 43 * 60) as f32, &mut out);
        assert_eq!(out, "91d 9h 43m ".to_string());
    }

    #[test]
    fn atom_comp_avg_no_history() {
        let mut over = Over::NO;
        let mut atom = setup_atom(0);
        atom.last_time = (current_time() - 1) as u32;
        assert_eq!(atom.comp_avg(&mut over), 1.);
        assert!(matches!(over, Over::ALL));
    }

    #[test]
    fn atom_comp_avg_over_all() {
        let mut over = Over::NO;
        // 52h 8m ago
        let time = 52 * 60 * 60 + 8 * 60;
        let mut atom = setup_atom(21);
        atom.last_time = (current_time() - time) as u32;
        assert_eq!(atom.comp_avg(&mut over), (time - 21) as f32);
        assert!(matches!(over, Over::ALL));
    }

    #[test]
    fn atom_comp_avg_over_avg() {
        let mut over = Over::NO;
        let mut atom = setup_atom(10);
        atom.add(10);
        atom.add(61);
        atom.last_time = (current_time() - 15) as u32;
        assert_eq!(atom.comp_avg(&mut over), 12. * 1.25 + 60.);
        assert!(matches!(over, Over::AVG));
    }

    #[test]
    fn atom_comp_avg_over_no() {
        let mut over = Over::NO;
        let mut atom = setup_atom(60);
        atom.last_time = (current_time() - 10) as u32;
        assert_eq!(atom.comp_avg(&mut over), 50. * 1.25 + 60.);
        assert!(matches!(over, Over::NO));
    }

    #[test]
    fn atom_return_time_work() {
        let mut time = String::new();
        let mut atom = setup_atom(10);
        atom.last_time = current_time() as u32;

        assert!(matches!(atom.return_time(&mut time), Over::NO));
        assert_eq!(time, "1m ");
    }
}
