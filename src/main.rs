use std::{collections::HashMap, process};

use clap::Parser;
use genlogsum;

fn emerge_file(
    file: &String,
    config: &genlogsum::Arguments,
) -> Result<(), Box<dyn std::error::Error>> {
    let emerges_not_complete: &mut HashMap<String, genlogsum::package::PackageInfo> =
        &mut HashMap::new();
    let completed_atoms: &mut HashMap<String, genlogsum::package::Atom> = &mut HashMap::new();

    genlogsum::read_file(&file, emerges_not_complete, completed_atoms)?;
    genlogsum::set_last_time(emerges_not_complete, completed_atoms);

    if emerges_not_complete.is_empty() {
        println!("Not currently emerging");
    } else {
        for emerge in emerges_not_complete.values() {
            println!(
                "{}",
                genlogsum::status_package(emerge, completed_atoms, config)
                    .unwrap_or("".to_string())
            );
        }
    }

    return Ok(());
}

fn main() {
    let args = &genlogsum::Arguments::parse();

    for file in &args.files {
        if let Err(e) = emerge_file(file, args) {
            eprintln!("Application error: {e}");
            process::exit(1);
        }
    }
}
