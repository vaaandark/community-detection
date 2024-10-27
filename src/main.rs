#![allow(internal_features)]
#![feature(slice_internals)]
#![feature(slice_split_once)]

use std::{env, process::exit};

use read::to_sorted_edges;

mod graph;
mod read;
mod timer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} <input_data_file>", args.first().unwrap());
        exit(1)
    }
    let filename = &args[1];

    let edges = to_sorted_edges(filename);
    println!("edge size: {}", edges.len());
}
