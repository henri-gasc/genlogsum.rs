use serde_json::Value;
use std::fs;

pub struct EmergeResume {
    pub binary: bool,
    // root: String,
    pub name: String,
    // action: String,
}

impl EmergeResume {
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

    let resume = match parsed.get("resume") {
        Some(val) => val,
        None => return vec![],
    };

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
