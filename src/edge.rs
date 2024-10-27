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
