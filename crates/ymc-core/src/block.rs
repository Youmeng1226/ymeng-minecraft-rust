/// Every block type the engine knows about.
///
/// Stored as a single byte so a chunk stays compact in memory.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Block {
    #[default]
    Air = 0,
    Stone,
    Dirt,
    Grass,
    Sand,
    Water,
    Bedrock,
    OakLog,
    OakLeaves,
}

impl Block {
    pub const ALL: [Block; 9] = [
        Block::Air,
        Block::Stone,
        Block::Dirt,
        Block::Grass,
        Block::Sand,
        Block::Water,
        Block::Bedrock,
        Block::OakLog,
        Block::OakLeaves,
    ];

    pub fn id(self) -> u8 {
        self as u8
    }

    pub fn from_id(id: u8) -> Option<Block> {
        Self::ALL.get(id as usize).copied()
    }

    pub fn name(self) -> &'static str {
        match self {
            Block::Air => "air",
            Block::Stone => "stone",
            Block::Dirt => "dirt",
            Block::Grass => "grass_block",
            Block::Sand => "sand",
            Block::Water => "water",
            Block::Bedrock => "bedrock",
            Block::OakLog => "oak_log",
            Block::OakLeaves => "oak_leaves",
        }
    }

    /// Whether the block occupies its full cube and hides neighbouring faces.
    pub fn is_opaque(self) -> bool {
        !matches!(self, Block::Air | Block::Water | Block::OakLeaves)
    }

    /// Whether the block takes part in meshing at all.
    pub fn is_visible(self) -> bool {
        self != Block::Air
    }

    /// Whether players and entities collide with the block.
    pub fn is_solid(self) -> bool {
        !matches!(self, Block::Air | Block::Water)
    }

    /// Texture atlas index for the given face.
    pub fn texture_index(self, face: BlockFace) -> u16 {
        match (self, face) {
            (Block::Grass, BlockFace::Top) => 0,
            (Block::Grass, BlockFace::Bottom) => 2,
            (Block::Grass, _) => 1,
            (Block::Dirt, _) => 2,
            (Block::Stone, _) => 3,
            (Block::Sand, _) => 4,
            (Block::Water, _) => 5,
            (Block::Bedrock, _) => 6,
            (Block::OakLog, BlockFace::Top | BlockFace::Bottom) => 8,
            (Block::OakLog, _) => 7,
            (Block::OakLeaves, _) => 9,
            (Block::Air, _) => u16::MAX,
        }
    }
}

/// The six faces of a cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl BlockFace {
    pub const ALL: [BlockFace; 6] = [
        BlockFace::Top,
        BlockFace::Bottom,
        BlockFace::North,
        BlockFace::South,
        BlockFace::East,
        BlockFace::West,
    ];

    /// Unit offset pointing out of the face.
    pub fn normal(self) -> [i32; 3] {
        match self {
            BlockFace::Top => [0, 1, 0],
            BlockFace::Bottom => [0, -1, 0],
            BlockFace::North => [0, 0, -1],
            BlockFace::South => [0, 0, 1],
            BlockFace::East => [1, 0, 0],
            BlockFace::West => [-1, 0, 0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_round_trip() {
        for block in Block::ALL {
            assert_eq!(Block::from_id(block.id()), Some(block));
        }
        assert_eq!(Block::from_id(200), None);
    }

    #[test]
    fn air_is_invisible() {
        assert!(!Block::Air.is_visible());
        assert!(!Block::Air.is_opaque());
        assert!(Block::Stone.is_opaque());
    }
}
