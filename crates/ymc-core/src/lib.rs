//! Core data types shared by every part of YMeng Minecraft.

pub mod block;
pub mod chunk;
pub mod coords;

pub use block::{Block, BlockFace};
pub use chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE, CHUNK_VOLUME};
pub use coords::{BlockPos, ChunkPos, LocalPos};
