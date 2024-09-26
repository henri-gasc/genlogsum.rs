use std::u32;

/// A structure to store the data until we find a line that allows us to either discard it, or add it to the list of Atoms
pub struct PackageInfo {
    pub category: String,  // the category of the package
    pub name: String,      // the package name
    pub full_name: String, // the full name of the package (including version, revision, status)
    pub time: u32,         // The complete line in the file
    pub is_binary: bool,   // is it a binary emerge
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

impl Default for Atom {
    fn default() -> Self {
        return Self {
            cpn: "".to_string(),
            num_emerge: 0,
            total_time: 0,
            best_time: u32::MAX,
            worst_time: 0,
            last_time: 0,
        };
    }
}

impl Atom {
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

    pub fn add(&mut self, time: u32) {
        self.num_emerge += 1;
        self.total_time += time;
        self.worst_time = std::cmp::max(self.worst_time, time);
        self.best_time = std::cmp::min(self.best_time, time);
    }
}

//  /**
//   * @brief Compute the average time with filter
//   *
//   * @return float The average time
//   */
//  float filter_time(void);

//  /**
//   * @brief Return the average time for an emerge
//   *
//   * @return float the average time
//   */
//  float time_avg(void);

//  /**
//   * @brief Compute the average time for the emerge, along with the filters
//   * needed
//   *
//   * @param over Where to store the comparaison result (know if time diff is <
//   * or > worst_time)
//   * @return long int The average
//   */
//  long int comp_avg(int &over);

//  /**
//   * @brief Format time to be on the format d h m, or with other special text
//   *
//   * @param time The time you want to convert
//   * @return std::string The time converted
//   */
//  std::string convert_text(long int time);

//  /**
//   * @brief Format the time of Atom to "xd yh zm" format with special format
//   * should avg - diff yield a negative result
//   *
//   * @param over set over to the correct value of OVER_*
//   * @return std::string The time formatted
//   */
//  std::string return_time(int &over);

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
    }
}
