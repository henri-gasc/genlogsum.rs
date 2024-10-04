use std::collections::HashMap;

use clap::Parser;
use genlogsum;

fn emerge_file(
    file: &str,
    config: &genlogsum::Arguments,
    fakeroot: &str,
    print: &mut String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut emerges_not_complete: HashMap<String, genlogsum::PackageInfo> = HashMap::new();
    let mut completed_atoms: HashMap<String, genlogsum::Atom> = HashMap::new();

    genlogsum::read_file(&file, &mut emerges_not_complete, &mut completed_atoms)?;
    genlogsum::set_last_time(&emerges_not_complete, &mut completed_atoms);

    if !config.all {
        for package in emerges_not_complete.values() {
            genlogsum::emerge_package(package, &completed_atoms, config, fakeroot, print);
        }
    } else {
        // Create next_emerge from data from mtimedb
        let list = genlogsum::read_mtimedb(fakeroot);
        for p in list {
            genlogsum::emerge_package_mtimedb(&p, &mut completed_atoms, print);
        }
    }

    return Ok(());
}

fn emerge_fakeroot(fakeroot: &str, config: &genlogsum::Arguments, print: &mut String) {
    for file in &config.files {
        let mut path = String::new();
        genlogsum::correct_path(fakeroot, file, &mut path);
        if let Err(e) = emerge_file(&path, config, fakeroot, print) {
            if !config.skip_file {
                eprintln!("Application error: {e} for {path}");
            }
        }
    }
}

fn main() {
    let args = &genlogsum::Arguments::parse();
    let mut print = String::new();

    for fakeroot in &args.fakeroots {
        emerge_fakeroot(fakeroot, args, &mut print);
    }

    if print.is_empty() {
        println!("Not currently emerging");
    } else {
        // There is a newline at the end of print
        print!("{print}");
    }
}
