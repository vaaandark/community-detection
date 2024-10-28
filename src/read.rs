use crate::{
    edge::{Edge, EdgeAsInt},
    timer::Timer,
};
use core::slice::memchr;
use std::{fs::File, sync::Mutex};
use voracious_radix_sort::RadixSort;

const CHARS_PER_LINE: usize = 20;

pub fn to_sorted_edges(filename: &str) -> Vec<u64> {
    let _timer = Timer::with_label("reading file");

    let file = File::open(filename).unwrap();
    let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };
    let content = &mmap[..];
    let file_size = content.len();

    let nthreads = rayon::current_num_threads();
    let mut splitting_positions = Vec::with_capacity(nthreads + 1);

    {
        let _timer = Timer::with_label("splitting file into slices");
        splitting_positions.push(0);
        for i in 1..nthreads {
            let begin = file_size / nthreads * i;
            let offset = memchr::memchr(b'\n', &content[begin..]).unwrap();
            splitting_positions.push(begin + offset);
        }
        splitting_positions.push(file_size);
    }

    fn read_slice(slice: &[u8]) -> Vec<EdgeAsInt> {
        let mut result = Vec::with_capacity(slice.len() / CHARS_PER_LINE);
        let extend = slice
            .split(|&x| x == b'\n')
            .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
            .filter_map(|line| line.split_once(|x| x.is_ascii_whitespace()))
            .filter_map(|(from, to)| {
                let from: u32 = atoi_simd::parse(from).ok()?;
                let to: u32 = atoi_simd::parse(to).ok()?;
                Some((from, to))
            })
            .flat_map(|(from, to)| {
                vec![
                    EdgeAsInt::from(Edge(to, from)),
                    EdgeAsInt::from(Edge(from, to)),
                ]
            });
        result.extend(extend);
        result
    }

    let result = Mutex::new(Vec::new());
    let edge_slices = {
        rayon::scope(|s| {
            let _timer = Timer::with_label("parallel parsing file");
            for i in 0..nthreads {
                let start = splitting_positions[i];
                let end = splitting_positions[i + 1];
                let slice = &content[start..end];
                s.spawn(|_| {
                    let edges = read_slice(slice);
                    result.lock().unwrap().push(edges);
                });
            }
        });
        result.into_inner().unwrap()
    };

    let edges = {
        let _timer = Timer::with_label("merging vertices");
        let total_len = edge_slices.iter().map(Vec::len).sum::<usize>();
        let mut result = Vec::with_capacity(total_len);
        #[allow(clippy::uninit_vec)]
        unsafe {
            result.set_len(total_len)
        };
        let mut buf = &mut result[..];
        rayon::scope(|s| {
            for item in edge_slices.into_iter() {
                let (current, res) = buf.split_at_mut(item.len());
                buf = res;
                s.spawn(move |_| current.copy_from_slice(item.as_slice()));
            }
        });
        result
    };

    {
        let _timer = Timer::with_label("sorting");
        let mut result = edges;
        result.voracious_mt_sort(rayon::current_num_threads());
        result
    }
}
