use crate::block::Block;
use crate::coords::{ChunkPos, LocalPos};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 128;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT;

/// A 16 x 128 x 16 column of blocks.
#[derive(Clone)]
pub struct Chunk {
    pos: ChunkPos,
    blocks: Box<[Block; CHUNK_VOLUME]>,
    dirty: bool,
}

impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            blocks: Box::new([Block::Air; CHUNK_VOLUME]),
            dirty: true,
        }
    }

    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_HEIGHT && z < CHUNK_SIZE);
        (y * CHUNK_SIZE + z) * CHUNK_SIZE + x
    }

    #[inline]
    pub fn get(&self, pos: LocalPos) -> Block {
        self.blocks[Self::index(pos.x, pos.y, pos.z)]
    }

    /// Looks up a block using signed local coordinates.
    /// Anything outside the chunk is reported as `None`.
    pub fn get_signed(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        let in_range = |v: i32, max: usize| v >= 0 && (v as usize) < max;
        if in_range(x, CHUNK_SIZE) && in_range(y, CHUNK_HEIGHT) && in_range(z, CHUNK_SIZE) {
            Some(self.blocks[Self::index(x as usize, y as usize, z as usize)])
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: LocalPos, block: Block) {
        let idx = Self::index(pos.x, pos.y, pos.z);
        if self.blocks[idx] != block {
            self.blocks[idx] = block;
            self.dirty = true;
        }
    }

    /// Highest non-air block in a column.
    pub fn height_at(&self, x: usize, z: usize) -> Option<usize> {
        (0..CHUNK_HEIGHT)
            .rev()
            .find(|&y| self.get(LocalPos::new(x, y, z)) != Block::Air)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn count(&self, block: Block) -> usize {
        self.blocks.iter().filter(|&&b| b == block).count()
    }

    /// Run-length encodes the chunk. This is the planned on-disk format.
    pub fn encode_rle(&self) -> Vec<(u8, u16)> {
        let mut runs: Vec<(u8, u16)> = Vec::new();
        for block in self.blocks.iter() {
            match runs.last_mut() {
                Some((id, len)) if *id == block.id() && *len < u16::MAX => *len += 1,
                _ => runs.push((block.id(), 1)),
            }
        }
        runs
    }

    pub fn decode_rle(pos: ChunkPos, runs: &[(u8, u16)]) -> Option<Self> {
        let mut chunk = Chunk::new(pos);
        let mut i = 0;
        for &(id, len) in runs {
            let block = Block::from_id(id)?;
            let end = i + len as usize;
            if end > CHUNK_VOLUME {
                return None;
            }
            chunk.blocks[i..end].fill(block);
            i = end;
        }
        (i == CHUNK_VOLUME).then_some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        let pos = LocalPos::new(3, 40, 7);
        chunk.set(pos, Block::Stone);
        assert_eq!(chunk.get(pos), Block::Stone);
        assert_eq!(chunk.height_at(3, 7), Some(40));
        assert_eq!(chunk.height_at(0, 0), None);
    }

    #[test]
    fn rle_round_trip() {
        let mut chunk = Chunk::new(ChunkPos::new(1, -1));
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set(LocalPos::new(x, 0, z), Block::Bedrock);
                chunk.set(LocalPos::new(x, 1, z), Block::Stone);
            }
        }
        let runs = chunk.encode_rle();
        assert!(runs.len() < 10);
        let decoded = Chunk::decode_rle(chunk.pos(), &runs).unwrap();
        assert_eq!(decoded.count(Block::Bedrock), CHUNK_SIZE * CHUNK_SIZE);
        assert_eq!(decoded.count(Block::Stone), CHUNK_SIZE * CHUNK_SIZE);
    }

    #[test]
    fn signed_lookup_out_of_bounds() {
        let chunk = Chunk::new(ChunkPos::new(0, 0));
        assert_eq!(chunk.get_signed(-1, 0, 0), None);
        assert_eq!(chunk.get_signed(0, CHUNK_HEIGHT as i32, 0), None);
        assert_eq!(chunk.get_signed(0, 0, 0), Some(Block::Air));
    }
}
