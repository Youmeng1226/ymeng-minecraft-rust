use std::collections::HashMap;

use ymc_core::{Block, BlockPos, Chunk, ChunkPos};
use ymc_mesh::{build_mesh, ChunkMesh, NeighborLookup};
use ymc_worldgen::TerrainGenerator;

/// Loaded chunks plus the generator used to create new ones.
pub struct World {
    generator: TerrainGenerator,
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new(generator: TerrainGenerator) -> Self {
        Self {
            generator,
            chunks: HashMap::new(),
        }
    }

    pub fn load_area(&mut self, center: ChunkPos, radius: i32) {
        for pos in center.spiral(radius) {
            self.chunks
                .entry(pos)
                .or_insert_with(|| self.generator.generate(pos));
        }
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn block(&self, pos: BlockPos) -> Option<Block> {
        let local = pos.local()?;
        self.chunks.get(&pos.chunk()).map(|c| c.get(local))
    }

    pub fn mesh_chunk(&self, pos: ChunkPos) -> Option<ChunkMesh> {
        let chunk = self.chunks.get(&pos)?;
        Some(build_mesh(
            chunk,
            &WorldNeighbors {
                world: self,
                chunk: pos,
            },
        ))
    }

    pub fn generator(&self) -> &TerrainGenerator {
        &self.generator
    }
}

struct WorldNeighbors<'a> {
    world: &'a World,
    chunk: ChunkPos,
}

impl NeighborLookup for WorldNeighbors<'_> {
    fn block_outside(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        let origin = self.chunk.origin();
        self.world
            .block(BlockPos::new(origin.x + x, y, origin.z + z))
    }
}
