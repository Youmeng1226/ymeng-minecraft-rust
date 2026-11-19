use crate::chunk::{CHUNK_HEIGHT, CHUNK_SIZE};

/// Absolute block position in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Horizontal position of a chunk column, measured in chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Position of a block inside a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn chunk(self) -> ChunkPos {
        let size = CHUNK_SIZE as i32;
        ChunkPos {
            x: self.x.div_euclid(size),
            z: self.z.div_euclid(size),
        }
    }

    /// Returns `None` when `y` is outside the world height.
    pub fn local(self) -> Option<LocalPos> {
        if self.y < 0 || self.y >= CHUNK_HEIGHT as i32 {
            return None;
        }
        let size = CHUNK_SIZE as i32;
        Some(LocalPos {
            x: self.x.rem_euclid(size) as usize,
            y: self.y as usize,
            z: self.z.rem_euclid(size) as usize,
        })
    }
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// World position of the chunk's (0, 0, 0) block.
    pub fn origin(self) -> BlockPos {
        BlockPos::new(self.x * CHUNK_SIZE as i32, 0, self.z * CHUNK_SIZE as i32)
    }

    /// All chunk positions within a square radius, nearest first.
    pub fn spiral(self, radius: i32) -> Vec<ChunkPos> {
        let mut out = Vec::new();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                out.push(ChunkPos::new(self.x + dx, self.z + dz));
            }
        }
        out.sort_by_key(|p| (p.x - self.x).pow(2) + (p.z - self.z).pow(2));
        out
    }
}

impl LocalPos {
    pub const fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn to_world(self, chunk: ChunkPos) -> BlockPos {
        let origin = chunk.origin();
        BlockPos::new(
            origin.x + self.x as i32,
            self.y as i32,
            origin.z + self.z as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_coordinates_map_to_correct_chunk() {
        let pos = BlockPos::new(-1, 10, -17);
        assert_eq!(pos.chunk(), ChunkPos::new(-1, -2));
        assert_eq!(pos.local(), Some(LocalPos::new(15, 10, 15)));
    }

    #[test]
    fn local_world_round_trip() {
        let pos = BlockPos::new(-37, 64, 99);
        let local = pos.local().unwrap();
        assert_eq!(local.to_world(pos.chunk()), pos);
    }

    #[test]
    fn spiral_starts_at_center() {
        let center = ChunkPos::new(3, -2);
        let ring = center.spiral(2);
        assert_eq!(ring.len(), 25);
        assert_eq!(ring[0], center);
    }
}
