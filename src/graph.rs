use std::collections::{HashMap, HashSet};

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct Edge(pub u32, pub u32);

pub type EdgeAsInt = u64;

impl From<Edge> for EdgeAsInt {
    #[inline(always)]
    fn from(value: Edge) -> Self {
        ((value.0 as EdgeAsInt) << 32) | (value.1 as EdgeAsInt)
    }
}

impl From<EdgeAsInt> for Edge {
    #[inline(always)]
    fn from(value: EdgeAsInt) -> Self {
        Self((value >> 32) as u32, value as u32)
    }
}

impl From<&EdgeAsInt> for Edge {
    #[inline(always)]
    fn from(value: &EdgeAsInt) -> Self {
        Self::from(*value)
    }
}

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
