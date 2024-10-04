use std::collections::HashMap;

use clap::Parser;
use genlogsum;

fn emerge_file(
    file: &str,
    config: &genlogsum::Arguments,
    fakeroot: &str,
    print: &mut String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut emerges_not_complete: HashMap<String, genlogsum::package::PackageInfo> = HashMap::new();
    let mut completed_atoms: HashMap<String, genlogsum::package::Atom> = HashMap::new();

    genlogsum::read_file(&file, &mut emerges_not_complete, &mut completed_atoms)?;
    genlogsum::set_last_time(&emerges_not_complete, &mut completed_atoms);

    if !emerges_not_complete.is_empty() {
        for package in emerges_not_complete.values() {
            let mut out = String::new();
            if config.show_root && (fakeroot != "/") {
                let name = std::path::Path::new(fakeroot).components().next_back();
                if let Some(val) = name {
                    out.push_str(val.as_os_str().to_str().unwrap_or(""));
                    out.push_str(": ");
                }
            }
            out.push_str(
                &genlogsum::status_package(package, &mut completed_atoms, config, fakeroot)
                    .unwrap_or("".to_string()),
            );

            print.push_str(&format!("{out}\n"));
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
    }
}
