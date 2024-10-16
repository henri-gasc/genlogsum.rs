#![allow(dead_code)]

//! Benchmark code for options
//!
//! Code to benchmark the multiple ways I can do something  
//! To use this, you should create a file with garbage data at ../tests/garbage_data (use `scripts/generate_garbage_data.py`).  
//! To run this code, you should run `scripts/run_benchmark.sh`.

// // // // // // // // // // // // // // // // // // // // // // // //
//
// genlogsum: GENtoo LOG SUMmary, summarize log to show running emerge
// Copyright (C) 2024 Henri GASC
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
// // // // // // // // // // // // // // // // // // // // // // // //

const LITTLE_COUNT: u32 = 1_000;
const MIDDLE_COUNT: u32 = 100_000;
const HUGE_COUNT: u32 = 100_000_000;

fn main() {
    let string = std::fs::read_to_string("./garbage_data").unwrap();
    let text = string.as_str();

    println!("string.find():");
    string_find(text);

    println!("text.starts_with():");
    starts_with(text);

    println!("text.ends_with():");
    ends_with(text);

    println!("format! inline or to_string:");
    to_string_or_inline();
}

fn string_find(text: &str) {
    for _ in 0..LITTLE_COUNT {
        core::hint::black_box(text.find('c'));
    }

    let char = std::time::Instant::now();
    for _ in 0..LITTLE_COUNT {
        core::hint::black_box(text.find('c'));
    }
    println!("\tchar: {:?}", char.elapsed());

    let str_address = std::time::Instant::now();
    for _ in 0..LITTLE_COUNT {
        core::hint::black_box(text.find("c"));
    }
    println!("\tstr_address: {:?}", str_address.elapsed());
}

fn starts_with(text: &str) {
    for _ in 0..HUGE_COUNT {
        core::hint::black_box(text.starts_with('<'));
    }

    let str_address = std::time::Instant::now();
    for _ in 0..HUGE_COUNT {
        core::hint::black_box(text.starts_with("<"));
    }
    println!("\tstr_address: {:?}", str_address.elapsed());

    let char = std::time::Instant::now();
    for _ in 0..HUGE_COUNT {
        core::hint::black_box(text.starts_with('<'));
    }
    println!("\tchar: {:?}", char.elapsed());
}

fn ends_with(text: &str) {
    for _ in 0..HUGE_COUNT {
        core::hint::black_box(text.ends_with('<'));
    }

    let str_address = std::time::Instant::now();
    for _ in 0..HUGE_COUNT {
        core::hint::black_box(text.ends_with("<"));
    }
    println!("\tstr_address: {:?}", str_address.elapsed());

    let char = std::time::Instant::now();
    for _ in 0..HUGE_COUNT {
        core::hint::black_box(text.ends_with('<'));
    }
    println!("\tchar: {:?}", char.elapsed());
}

fn to_string_or_inline() {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time was warped. Fix it !")
        .as_nanos();

    let to_string = std::time::Instant::now();
    for _ in 0..MIDDLE_COUNT {
        core::hint::black_box(format!("coucou: {}", time.to_string()));
    }
    println!("\tto_string: {:?}", to_string.elapsed());

    let inline = std::time::Instant::now();
    for _ in 0..MIDDLE_COUNT {
        core::hint::black_box(format!("coucou: {time}"));
    }
    println!("\tinline: {:?}", inline.elapsed());
}
