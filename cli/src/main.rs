use std::{env, path::PathBuf};

use common::{default_save_filename, read_events};

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath: PathBuf = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        default_save_filename()
    };
    let events = read_events(filepath).unwrap();
    println!("{events:?}");
}
