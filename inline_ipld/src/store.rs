//! Content addressed storage

mod memory;
mod traits;

pub use memory::MemoryStore;
pub use traits::{GetRawError, Store};
