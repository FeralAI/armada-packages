//! RGB lighting support for Armada devices.

mod backend;
mod config;
mod controller;
mod runtime;
mod state;

pub use backend::{LightingBackend, MulticolorBackend};
pub use controller::Controller;
pub use state::LightingConfig;
