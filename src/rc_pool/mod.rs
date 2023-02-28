pub use pool::*;
pub use strong_ref::*;
pub use weak_ref::*;

mod page;
mod pool;
mod slot;
mod strong_ref;
mod weak_ref;

pub type Index = u32;
pub type Version = usize; // Reference + version is two machine words
pub type Count = u32;
