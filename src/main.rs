use std::process;

use genlogsum;

fn main() {
    if let Err(e) = genlogsum::read_file() {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
