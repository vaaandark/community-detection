use std::{collections::HashMap, sync::Mutex};

use crate::{
    edge::{Edge, EdgeAsInt},
    timer::Timer,
};

const DEGREES_PER_VERTEX: usize = 4;

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub struct Vertex {
    pub id: u32,
    pub community: u32,
    pub neighbors: HashMap<u32, u32>,
}

impl Vertex {
    #[allow(unused)]
    pub fn new(id: u32) -> Self {
        Self {
            id,
            community: id,
            ..Default::default()
        }
    }

    #[allow(unused)]
    pub fn add_neighbor(&mut self, neighbor: u32, weight: u32) {
        self.neighbors.insert(neighbor, weight);
    }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Community {
    pub id: u32,
    pub vertices: HashMap<u32, Vertex>,
}

impl Community {
    #[allow(unused)]
    fn from_single_vertex(vertex: Vertex) -> Self {
        let id = vertex.id;
        let mut vertices = HashMap::new();
        vertices.insert(id, vertex);
        Self { id, vertices }
    }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Graph {
    pub communities: HashMap<u32, Community>,
}

impl Graph {
    #[allow(unused)]
    pub fn from_edges(edges: &[EdgeAsInt]) -> Self {
        let _timer = Timer::with_label("building graph");

        let nedges = edges.len();
        let nthreads = rayon::current_num_threads();
        let mut splitting_positions = Vec::with_capacity(nthreads + 1);

        {
            let _timer = Timer::with_label("splitting edges into slices");
            splitting_positions.push(0);
            for i in 1..nthreads {
                let begin = nedges / nthreads * i;
                let mut real_begin = begin;
                for j in (begin + 1)..nedges {
                    let from1 = Edge::from(edges[j]).0;
                    let from2 = Edge::from(edges[j - 1]).0;
                    if from1 != from2 {
                        real_begin = j;
                        break;
                    }
                }
                splitting_positions.push(real_begin);
            }
            splitting_positions.push(nedges);
        }

        fn build_community_slice(slice: &[EdgeAsInt]) -> HashMap<u32, Community> {
            let mut result = HashMap::with_capacity(slice.len() / DEGREES_PER_VERTEX);
            for edge in slice {
                let edge = Edge::from(edge);
                let (from, to) = (edge.0, edge.1);
                let community = result
                    .entry(from)
                    .or_insert(Community::from_single_vertex(Vertex::new(from)));
                community
                    .vertices
                    .get_mut(&from)
                    .unwrap()
                    .add_neighbor(to, 1);
            }
            result
        }

        let result = Mutex::new(Vec::new());
        let mut community_clices = {
            rayon::scope(|s| {
                let _timer = Timer::with_label("parallel building graphs");
                for i in 0..nthreads {
                    let start = splitting_positions[i];
                    let end = splitting_positions[i + 1];
                    let slice = &edges[start..end];
                    s.spawn(|_| {
                        let edges = build_community_slice(slice);
                        result.lock().unwrap().push(edges);
                    });
                }
            });
            result.into_inner().unwrap()
        };

        let communities = {
            let _timer = Timer::with_label("merging graph slices");
            let total_len = community_clices.iter().map(HashMap::len).sum::<usize>();
            let mut result = HashMap::with_capacity(total_len);
            for slice in community_clices.iter_mut() {
                result.extend(slice.drain());
            }
            result
        };

        Graph { communities }
    }
}
