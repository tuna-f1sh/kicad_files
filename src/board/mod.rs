//! **Board Common Syntax**
//!
//! This module defines all syntax that is shared across the footprint library and
//! printed circuit board file formats.

mod connect_pads;
pub mod footprint;
mod footprint_module;
pub mod graphic;
mod layer;
mod timestamp;
pub mod pcb;

pub use connect_pads::ConnectPads;
pub use footprint::Footprint;
pub use layer::Layer;
pub use timestamp::Timestamp;
