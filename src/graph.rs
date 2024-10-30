use std::{
    collections::{HashMap, HashSet}, io::Write, sync::Mutex
};

use crate::{
    edge::{Edge, EdgeAsInt},
    timer::Timer,
};

const DEGREES_PER_VERTEX: usize = 20;

const CONVERGENCE_THRESHOLD: f64 = 0.001;

const MAX_INNER_ITERS: usize = 10;

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub struct Vertex {
    pub id: u32,
    pub community: u32,
    pub neighbors: HashMap<u32, usize>,
    degrees: usize,
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

    pub fn degrees(&self) -> usize {
        self.degrees
    }

    #[allow(unused)]
    pub fn add_neighbor(&mut self, neighbor: u32, weight: usize) -> Option<usize> {
        self.degrees += weight;
        self.neighbors.insert(neighbor, weight)
    }

    #[allow(unused)]
    pub fn add_neighbor_or_accumulate(&mut self, neighbor: u32, mut weight: usize) -> usize {
        self.degrees += weight;
        if let Some(old) = self.neighbors.get(&neighbor) {
            weight += old;
        }
        let _ = self.neighbors.insert(neighbor, weight);
        weight
    }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Community {
    pub id: u32,
    pub vertices: HashSet<u32>,
    degrees: usize,
}

impl Community {
    #[allow(unused)]
    fn from_single_vertex(vertex: &Vertex) -> Self {
        let id = vertex.id;
        let degrees = vertex.degrees;
        let mut vertices = HashSet::new();
        vertices.insert(id);
        Self {
            id,
            vertices,
            degrees,
        }
    }

    #[allow(unused)]
    pub fn vertex<'a>(&self, graph: &'a Graph, vertex_id: u32) -> Option<&'a Vertex> {
        match self.vertices.get(&vertex_id) {
            Some(v) => graph.vertex(*v),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn vertices<'a: 'b, 'b>(&'a self, graph: &'b Graph) -> impl Iterator<Item = &'b Vertex> {
        self.vertices.iter().filter_map(|v| graph.vertex(*v))
    }

    pub fn remove(&mut self, vertex_id: u32, degrees: usize) -> bool {
        self.degrees -= degrees;
        self.vertices.remove(&vertex_id)
    }

    pub fn insert(&mut self, vertex_id: u32, degrees: usize) -> bool {
        self.degrees += degrees;
        self.vertices.insert(vertex_id)
    }

    pub fn degrees(&self) -> usize {
        self.degrees
    }

    #[allow(unused)]
    fn modularity(&self, graph: &Graph) -> f64 {
        let mut inner = 0;
        let mut outer = 0;
        for vertex in self.vertices(graph) {
            for (neighbor, weight) in &vertex.neighbors {
                if self.vertices.contains(neighbor) {
                    inner += weight;
                } else {
                    outer += weight;
                }
            }
        }
        let ec = (inner) as f64 / graph.total_degrees as f64;
        let ac = (inner + outer) as f64 / graph.total_degrees as f64;
        ec - ac * ac
    }

    #[allow(unused)]
    fn merge(&self, graph: &Graph) -> Option<(usize, Community, Vertex)> {
        if self.vertices.is_empty() {
            return None;
        }

        let mut merged_vertex = Vertex::new(self.id);

        let mut inner_degrees = 0;
        let mut total_degrees = 0;
        for vertex in self.vertices(graph) {
            for (neighbor, weight) in &vertex.neighbors {
                if self.vertices.contains(neighbor) {
                    inner_degrees += weight;
                } else {
                    let neigbor_community = graph.vertex(*neighbor).unwrap().community;
                    let _ = merged_vertex.add_neighbor_or_accumulate(neigbor_community, *weight);
                }
                total_degrees += weight;
            }
        }
        merged_vertex.add_neighbor(self.id, inner_degrees);

        Some((
            total_degrees,
            Community::from_single_vertex(&merged_vertex),
            merged_vertex,
        ))
    }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Graph {
    pub epoch: usize,
    pub total_degrees: usize,
    pub communities: HashMap<u32, Community>,
    vertices: HashMap<u32, Vertex>,
}

impl Graph {
    #[allow(unused)]
    pub fn community(&self, community_id: u32) -> Option<&Community> {
        self.communities.get(&community_id)
    }

    pub fn community_mut(&mut self, community_id: u32) -> Option<&mut Community> {
        self.communities.get_mut(&community_id)
    }

    pub fn communities(&self) -> impl Iterator<Item = &Community> {
        self.communities.values()
    }

    pub fn vertex(&self, vertex_id: u32) -> Option<&Vertex> {
        if let Some(v) = self.vertices.get(&vertex_id) {
            Some(v)
        } else {
            None
        }
    }

    pub fn vertex_mut(&mut self, vertex_id: u32) -> Option<&mut Vertex> {
        if let Some(v) = self.vertices.get_mut(&vertex_id) {
            Some(v)
        } else {
            None
        }
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Vertex> {
        self.vertices.values()
    }

    #[allow(unused)]
    pub fn from_edges(epoch: usize, edges: &[EdgeAsInt]) -> Self {
        let _timer = Timer::with_label("building graph");

        let nedges = edges.len();
        let nthreads = rayon::current_num_threads();
        let mut splitting_positions = Vec::with_capacity(nthreads + 1);

        {
            // let _timer = Timer::with_label("splitting edges into slices");
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

        fn build_community_slice(slice: &[EdgeAsInt]) -> (usize, HashMap<u32, Vertex>) {
            let mut result = HashMap::with_capacity(slice.len() / DEGREES_PER_VERTEX);
            let mut total_degrees = 0;
            for edge in slice {
                let edge = Edge::from(edge);
                let (from, to) = (edge.0, edge.1);
                let vertex = result.entry(from).or_insert(Vertex::new(from));
                let weight = if from == to { 2 } else { 1 };
                if vertex.add_neighbor(to, weight).is_none() {
                    total_degrees += weight;
                }
            }
            (total_degrees, result)
        }

        let result = Mutex::new(Vec::new());
        let mut vertex_slices = {
            rayon::scope(|s| {
                // let _timer = Timer::with_label("parallel building graphs");
                for i in 0..nthreads {
                    let start = splitting_positions[i];
                    let end = splitting_positions[i + 1];
                    let slice = &edges[start..end];
                    s.spawn(|_| {
                        let tuple = build_community_slice(slice);
                        result.lock().unwrap().push(tuple);
                    });
                }
            });
            result.into_inner().unwrap()
        };

        let mut total_degrees = 0;
        let vertices = {
            // let _timer = Timer::with_label("merging vertex slices");
            let total_len = vertex_slices
                .iter()
                .map(|tuple| tuple.1.len())
                .sum::<usize>();
            let mut result = HashMap::with_capacity(total_len);
            for (degrees, slice) in vertex_slices.iter_mut() {
                total_degrees += *degrees;
                result.extend(slice.drain());
            }
            result
        };

        let communities = vertices
            .values()
            .map(|v| (v.id, Community::from_single_vertex(v)))
            .collect();

        Graph {
            epoch,
            communities,
            total_degrees,
            vertices,
        }
    }

    #[allow(unused)]
    pub fn modularity(&self) -> f64 {
        self.communities().map(|c| c.modularity(self)).sum()
    }

    #[allow(unused)]
    pub fn merge(&self) -> Graph {
        let mut total_degrees = 0;
        let mut vertices = HashMap::new();
        let mut communities = HashMap::new();

        for old_community in self.communities() {
            if let Some((degrees, c, v)) = old_community.merge(self) {
                total_degrees += degrees;
                vertices.insert(v.id, v);
                communities.insert(c.id, c);
            }
        }

        Graph {
            epoch: self.epoch + 1,
            communities,
            total_degrees,
            vertices,
        }
    }

    fn modularity_gain(&self, vertex: &Vertex, neighbor_community_id: u32) -> Option<f64> {
        let neighbor_community = self.community(neighbor_community_id).unwrap();
        let vertex_degrees: usize = vertex.degrees();
        let mut community_degrees: usize = neighbor_community.degrees();
        let mut vertex_to_community_degrees: usize = vertex
            .neighbors
            .iter()
            .filter_map(|(neighbor, weight)| {
                if neighbor_community.vertices.contains(neighbor) {
                    Some(weight)
                } else {
                    None
                }
            })
            .sum();
        if vertex.community == neighbor_community_id {
            community_degrees -= vertex.degrees();
            vertex_to_community_degrees -= vertex.neighbors.get(&vertex.id).copied().unwrap_or(0);
        }
        let gain = vertex_to_community_degrees as f64
            - (community_degrees * vertex_degrees) as f64 / self.total_degrees as f64;
        if gain > 0.0 {
            Some(gain)
        } else {
            None
        }
    }

    fn max_modularity_gain(&self, vertex: &Vertex) -> Option<u32> {
        let mut max_gain = -1.0;
        let mut move_to = None;
        let mut calculated = HashSet::new();
        for &neighbor_vertex_id in vertex.neighbors.keys() {
            let neighbor_community_id = self.vertex(neighbor_vertex_id).unwrap().community;
            if calculated.contains(&neighbor_community_id) {
                continue;
            }
            let _ = calculated.insert(neighbor_community_id);
            if let Some(modularity_gain) = self.modularity_gain(vertex, neighbor_community_id) {
                if modularity_gain > max_gain {
                    max_gain = modularity_gain;
                    move_to = Some(neighbor_community_id);
                }
            }
        }
        move_to
    }

    fn move_vertex(&mut self, vertex_id: u32, dst_community_id: u32) {
        let vertex = self.vertex_mut(vertex_id).unwrap();
        let degrees = vertex.degrees();
        let src_community_id = vertex.community;
        vertex.community = dst_community_id;
        let old_community = self.community_mut(src_community_id).unwrap();
        old_community.remove(vertex_id, degrees);
        let dst_community = self.community_mut(dst_community_id).unwrap();
        dst_community.insert(vertex_id, degrees);
    }

    fn move_vertex_wrapper(graph: &Graph, vertex_id: u32, dst_community_id: u32) {
        #[allow(invalid_reference_casting)]
        let graph = unsafe { &mut *(graph as *const Graph as *mut Graph) };
        graph.move_vertex(vertex_id, dst_community_id);
    }

    #[allow(unused)]
    pub fn louvain(&mut self) -> (Graph, f64) {
        let mut last_modularity = self.modularity();
        let show_process = std::env::var("SHOW_PROCESS").ok().map(|s| s.parse::<bool>().unwrap_or(false)).unwrap_or(true);

        for i in 0..MAX_INNER_ITERS {
            let total = self.vertices.len();
            for (current, vertex) in self.vertices().enumerate() {
                if show_process {
                    print!("\r{}/{}", current + 1, total);
                    std::io::stdout().flush().unwrap();
                }
                if let Some(move_to) = self.max_modularity_gain(vertex) {
                    if vertex.community != move_to {
                    Self::move_vertex_wrapper(self, vertex.id, move_to);
                    }
                }
            }

            let modularity = self.modularity();
            if (modularity - last_modularity).abs() < CONVERGENCE_THRESHOLD {
                last_modularity = modularity;
                break;
            }
            last_modularity = modularity;
        }
        println!();

        (self.merge(), last_modularity)
    }
}
