use std::{collections::HashMap, process};

use genlogsum;

fn main() {
    let file = "/var/log/emerge.log";
    let emerges_not_complete: &mut HashMap<String, genlogsum::package::PackageInfo> =
        &mut HashMap::new();
    let completed_atoms: &mut HashMap<String, genlogsum::package::Atom> = &mut HashMap::new();

    if let Err(e) = genlogsum::read_file(file, emerges_not_complete, completed_atoms) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }

    genlogsum::set_last_time(emerges_not_complete, completed_atoms);

    if emerges_not_complete.is_empty() {
        println!("Not currently emerging");
    } else {
        for emerge in emerges_not_complete.values() {
            println!(
                "{}",
                genlogsum::status_package(emerge, completed_atoms).unwrap_or("".to_string())
            );
        }
    }
}
