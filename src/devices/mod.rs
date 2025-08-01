//! Device discovery and management for crab-dlna
//!
//! This module provides functionality for discovering and interacting with DLNA devices
//! on the network, including device discovery, render device management, and device types.

pub mod discovery;
pub mod render;
pub mod types;

// Re-export main types and functions for backward compatibility
pub use render::Render;
pub use types::{PositionInfo, RenderSpec, TransportInfo};
