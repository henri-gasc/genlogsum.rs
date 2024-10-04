#![warn(missing_docs)]

//! File for json (/var/cache/db/mtimedb) reading

use serde_json::Value;
use std::fs;

/// The kind of information we have in mtimedb, in the "resume" part
pub struct EmergeResume {
    /// If the type of package is binary
    pub binary: bool,
    // / What the root to merge the package is
    // root: String,
    /// The complete name of the package (with the version)
    pub name: String,
    // / What the action for this package is
    // action: String,
}

impl EmergeResume {
    /// Create a new EmergeResume from a Json value
    fn new(value: &Value) -> Self {
        let binary: bool = value[0] == "binary";
        // let root = String::from(
        //     value[1]
        //         .as_str()
        //         .expect("The second element should be the root"),
        // );
        let name = String::from(
            value[2]
                .as_str()
                .expect("The third element should be the name of the ebuild"),
        );
        // let action = String::from(
        //     value[3]
        //         .as_str()
        //         .expect("The fourth element should be the action"),
        // );

        return EmergeResume {
            binary,
            // root,
            name,
            // action,
        };
    }
}

/// Read mtimedb, extract the list of package that will be used next, and return it
///
/// * `root`: Where to start the path for mtimedb.  
///   By default the path is /var/cache/db/mtimedb
pub fn read_mtimedb(root: &str) -> Vec<EmergeResume> {
    let mut path = root.to_string();
    path.push_str("var/cache/edb/mtimedb");

    // Read file
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not read the file from root {root}: {e}");
            return vec![];
        }
    };

    // Parse file
    let parsed: Value = match serde_json::from_str(&content) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error while parsing the file: {e}");
            return vec![];
        }
    };

    // Load the resume section
    let resume = match parsed.get("resume") {
        Some(val) => val,
        None => return vec![],
    };

    // Load the mergelist
    let merge = match resume.get("mergelist") {
        Some(val) => val,
        None => return vec![],
    };

    return merge
        .as_array()
        .unwrap_or(&vec![Value::String("".to_string())])
        .iter()
        .map(|val: &Value| EmergeResume::new(val))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::read_mtimedb;

    #[test]
    fn read_resumelist() {
        read_mtimedb("/");
    }
}
