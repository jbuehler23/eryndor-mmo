// Allow common clippy warnings for game development
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::manual_unwrap_or_default)]
#![allow(clippy::single_match)]

pub mod ability_effects;
pub mod ability_types;
pub mod components;
pub mod protocol;
pub mod constants;

pub use ability_effects::*;
pub use ability_types::*;
pub use components::*;
pub use protocol::*;
pub use constants::*;
