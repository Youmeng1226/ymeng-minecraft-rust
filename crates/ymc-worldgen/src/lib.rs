//! Procedural terrain generation.

pub mod generator;
pub mod noise;

pub use generator::{TerrainGenerator, WorldGenConfig};
pub use noise::Perlin;
