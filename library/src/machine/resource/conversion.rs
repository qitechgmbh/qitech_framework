use std::fmt::Debug;

pub trait BoundedMeta { 
    type Bound: Copy + PartialOrd + Debug;
    fn as_bound(&self) -> Self::Bound;
}
