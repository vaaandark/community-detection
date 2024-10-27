use std::collections::{HashMap, HashSet};

use crate::edge::Edge;

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub struct Vertex {
    pub id: u32,
    pub neighbors: HashSet<u32>,
    // if it's none, the shortest path haven't been calculated yet
    shortest_path_len: Option<HashMap<u32, usize>>,
}

impl Vertex {
    #[allow(unused)]
    pub fn new(id: u32) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    #[allow(unused)]
    pub fn add_neighbor(&mut self, neighbor: u32) {
        self.shortest_path_len = None;
        self.neighbors.insert(neighbor);
    }

    #[allow(unused)]
    pub fn merge(&self, other: &Self) -> Self {
        if self.id != other.id {
            return self.clone();
        }
        Self {
            id: self.id,
            neighbors: self
                .neighbors
                .union(&other.neighbors)
                .copied()
                .collect::<HashSet<u32>>(),
            shortest_path_len: None,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Graph {
    pub vertices: HashMap<u32, Vertex>,
}

impl Graph {
    #[allow(unused)]
    pub fn from_edges(edges: &Vec<Edge>) -> Self {
        todo!()
    }
}
