#![allow(internal_features)]
#![feature(slice_internals)]
#![feature(slice_split_once)]

use std::{env, process::exit};

use graph::Graph;
use read::to_sorted_edges;

mod edge;
mod graph;
mod read;
mod timer;

const MAX_EPOCHS: usize = 20;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} <input_data_file>", args.first().unwrap());
        exit(1)
    }
    let filename = &args[1];

    let edges = to_sorted_edges(filename);
    let mut graph = Graph::from_edges(1, &edges);
    println!(
        "init: communities={}, degrees={}, modularity={}",
        graph.communities.len(),
        graph.total_degrees,
        graph.modularity()
    );

    let mut last_community_num = graph.communities.len();
    for epoch in 1..MAX_EPOCHS + 1 {
        let (graph_, modularity) = graph.louvain();
        graph = graph_;

        let community_num = graph.communities.len();
        let degrees = graph.total_degrees;

        println!(
            "epoch {}: communities={}, degrees={}, modularity={}",
            epoch, community_num, degrees, modularity
        );

        if community_num >= last_community_num {
            break;
        }
        last_community_num = community_num;
    }
}
