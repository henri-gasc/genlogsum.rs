use std::collections::HashMap;

use clap::Parser;
use genlogsum;

fn emerge_file(
    file: &str,
    config: &genlogsum::Arguments,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut emerges_not_complete: HashMap<String, genlogsum::package::PackageInfo> = HashMap::new();
    let mut completed_atoms: HashMap<String, genlogsum::package::Atom> = HashMap::new();

    genlogsum::read_file(&file, &mut emerges_not_complete, &mut completed_atoms)?;
    genlogsum::set_last_time(&emerges_not_complete, &mut completed_atoms);

    if emerges_not_complete.is_empty() {
        println!("Not currently emerging");
    } else {
        for emerge in emerges_not_complete.values() {
            println!(
                "{}",
                genlogsum::status_package(emerge, &mut completed_atoms, config)
                    .unwrap_or("".to_string())
            );
        }
    }

    return Ok(());
}

fn emerge_fakeroot(fakeroot: &str, config: &genlogsum::Arguments) {
    for file in &config.files {
        let mut path = String::new();
        genlogsum::correct_path(fakeroot, file, &mut path);
        if let Err(e) = emerge_file(&path, config) {
            if !config.skip_file {
                eprintln!("Application error: {e} for {path}");
            }
        }
    }
}

fn main() {
    let args = &genlogsum::Arguments::parse();

    for fakeroot in &args.fakeroots {
        emerge_fakeroot(fakeroot, args);
    }
}
