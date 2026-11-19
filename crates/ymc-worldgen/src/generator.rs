use ymc_core::{Block, Chunk, ChunkPos, LocalPos, CHUNK_HEIGHT, CHUNK_SIZE};

use crate::noise::{Perlin, SplitMix64};

#[derive(Debug, Clone)]
pub struct WorldGenConfig {
    pub seed: u64,
    pub sea_level: usize,
    pub base_height: f64,
    pub amplitude: f64,
    pub scale: f64,
    pub tree_chance: f64,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            seed: 20261119,
            sea_level: 48,
            base_height: 50.0,
            amplitude: 22.0,
            scale: 1.0 / 96.0,
            tree_chance: 0.012,
        }
    }
}

/// Heightmap-based terrain: stone, dirt, grass, beaches, water and trees.
pub struct TerrainGenerator {
    config: WorldGenConfig,
    height_noise: Perlin,
    detail_noise: Perlin,
}

impl TerrainGenerator {
    pub fn new(config: WorldGenConfig) -> Self {
        Self {
            height_noise: Perlin::new(config.seed),
            detail_noise: Perlin::new(config.seed.wrapping_mul(31).wrapping_add(17)),
            config,
        }
    }

    pub fn config(&self) -> &WorldGenConfig {
        &self.config
    }

    /// Surface height for a world column.
    pub fn height(&self, wx: i32, wz: i32) -> usize {
        let c = &self.config;
        let (x, z) = (wx as f64 * c.scale, wz as f64 * c.scale);
        let hills = self.height_noise.fbm(x, z, 5, 2.0, 0.5);
        let detail = self.detail_noise.get(x * 8.0, z * 8.0) * 0.08;
        let h = c.base_height + (hills + detail) * c.amplitude;
        h.clamp(1.0, (CHUNK_HEIGHT - 16) as f64) as usize
    }

    pub fn generate(&self, pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new(pos);
        let origin = pos.origin();
        let sea = self.config.sea_level;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = self.height(origin.x + x as i32, origin.z + z as i32);
                let beach = height <= sea + 1;

                for y in 0..=height.max(sea) {
                    let block = if y == 0 {
                        Block::Bedrock
                    } else if y > height {
                        Block::Water
                    } else if y + 4 < height {
                        Block::Stone
                    } else if beach {
                        Block::Sand
                    } else if y == height {
                        Block::Grass
                    } else {
                        Block::Dirt
                    };
                    chunk.set(LocalPos::new(x, y, z), block);
                }
            }
        }

        self.place_trees(&mut chunk);
        chunk
    }

    fn place_trees(&self, chunk: &mut Chunk) {
        let pos = chunk.pos();
        let seed = self.config.seed
            ^ (pos.x as u64).wrapping_mul(0x9E37_79B9)
            ^ (pos.z as u64).wrapping_mul(0x85EB_CA6B);
        let mut rng = SplitMix64(seed);

        // Keep trees two blocks away from chunk borders until
        // cross-chunk decoration is implemented.
        for x in 2..CHUNK_SIZE - 2 {
            for z in 2..CHUNK_SIZE - 2 {
                let roll = (rng.next() % 10_000) as f64 / 10_000.0;
                if roll >= self.config.tree_chance {
                    continue;
                }
                let Some(ground) = chunk.height_at(x, z) else {
                    continue;
                };
                if chunk.get(LocalPos::new(x, ground, z)) != Block::Grass {
                    continue;
                }
                let trunk = 4 + (rng.next() % 3) as usize;
                if ground + trunk + 3 >= CHUNK_HEIGHT {
                    continue;
                }
                Self::place_tree(chunk, x, ground + 1, z, trunk);
            }
        }
    }

    fn place_tree(chunk: &mut Chunk, x: usize, base: usize, z: usize, trunk: usize) {
        let top = base + trunk;
        for dy in 0..=2usize {
            let y = top - 1 + dy;
            let r: i32 = if dy == 2 { 1 } else { 2 };
            for dx in -r..=r {
                for dz in -r..=r {
                    if dx.abs() == r && dz.abs() == r {
                        continue;
                    }
                    let lx = (x as i32 + dx) as usize;
                    let lz = (z as i32 + dz) as usize;
                    let p = LocalPos::new(lx, y, lz);
                    if chunk.get(p) == Block::Air {
                        chunk.set(p, Block::OakLeaves);
                    }
                }
            }
        }
        for y in base..top {
            chunk.set(LocalPos::new(x, y, z), Block::OakLog);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic() {
        let gen = TerrainGenerator::new(WorldGenConfig::default());
        let a = gen.generate(ChunkPos::new(4, -9));
        let b = gen.generate(ChunkPos::new(4, -9));
        assert_eq!(a.encode_rle(), b.encode_rle());
    }

    #[test]
    fn bedrock_floor_everywhere() {
        let gen = TerrainGenerator::new(WorldGenConfig::default());
        let chunk = gen.generate(ChunkPos::new(0, 0));
        assert_eq!(chunk.count(Block::Bedrock), CHUNK_SIZE * CHUNK_SIZE);
    }

    #[test]
    fn columns_are_filled_up_to_sea_level() {
        let config = WorldGenConfig::default();
        let sea = config.sea_level;
        let gen = TerrainGenerator::new(config);
        let chunk = gen.generate(ChunkPos::new(-3, 2));
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                assert!(chunk.height_at(x, z).unwrap() >= sea);
            }
        }
    }
}
