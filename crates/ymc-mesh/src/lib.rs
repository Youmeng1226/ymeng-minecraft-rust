//! Turns chunks into GPU-ready vertex and index buffers.
//!
//! The current mesher emits one quad per visible face. Greedy meshing is
//! planned (see `docs/ROADMAP.md`).

use ymc_core::{Block, BlockFace, Chunk, LocalPos, CHUNK_HEIGHT, CHUNK_SIZE};

/// Vertex layout matching the planned `wgpu` shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub texture: u32,
}

#[derive(Debug, Default, Clone)]
pub struct ChunkMesh {
    pub opaque: MeshBuffers,
    pub transparent: MeshBuffers,
}

#[derive(Debug, Default, Clone)]
pub struct MeshBuffers {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    pub fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }

    fn push_face(&mut self, x: f32, y: f32, z: f32, face: BlockFace, texture: u32) {
        let base = self.vertices.len() as u32;
        let n = face.normal();
        let normal = [n[0] as f32, n[1] as f32, n[2] as f32];
        let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
        for (corner, uv) in face_corners(face).iter().zip(uvs) {
            self.vertices.push(Vertex {
                position: [x + corner[0], y + corner[1], z + corner[2]],
                normal,
                uv,
                texture,
            });
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

impl ChunkMesh {
    pub fn total_quads(&self) -> usize {
        self.opaque.quad_count() + self.transparent.quad_count()
    }
}

/// Neighbour lookup for blocks outside the chunk being meshed.
/// Returning `None` treats the neighbour as air.
pub trait NeighborLookup {
    fn block_outside(&self, x: i32, y: i32, z: i32) -> Option<Block>;
}

/// Treats everything outside the chunk as air.
pub struct NoNeighbors;

impl NeighborLookup for NoNeighbors {
    fn block_outside(&self, _: i32, _: i32, _: i32) -> Option<Block> {
        None
    }
}

pub fn build_mesh(chunk: &Chunk, neighbors: &impl NeighborLookup) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(LocalPos::new(x, y, z));
                if !block.is_visible() {
                    continue;
                }
                for face in BlockFace::ALL {
                    let [dx, dy, dz] = face.normal();
                    let (nx, ny, nz) = (x as i32 + dx, y as i32 + dy, z as i32 + dz);
                    let neighbor = chunk
                        .get_signed(nx, ny, nz)
                        .or_else(|| neighbors.block_outside(nx, ny, nz))
                        .unwrap_or(Block::Air);
                    if !face_visible(block, neighbor) {
                        continue;
                    }
                    let target = if block.is_opaque() {
                        &mut mesh.opaque
                    } else {
                        &mut mesh.transparent
                    };
                    let texture = block.texture_index(face) as u32;
                    target.push_face(x as f32, y as f32, z as f32, face, texture);
                }
            }
        }
    }
    mesh
}

fn face_visible(block: Block, neighbor: Block) -> bool {
    if neighbor.is_opaque() {
        return false;
    }
    // Don't draw faces between two blocks of the same transparent type
    // (e.g. water next to water).
    !(neighbor == block && !block.is_opaque())
}

fn face_corners(face: BlockFace) -> [[f32; 3]; 4] {
    match face {
        BlockFace::Top => [[0., 1., 1.], [1., 1., 1.], [1., 1., 0.], [0., 1., 0.]],
        BlockFace::Bottom => [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]],
        BlockFace::North => [[1., 0., 0.], [0., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
        BlockFace::South => [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
        BlockFace::East => [[1., 0., 1.], [1., 0., 0.], [1., 1., 0.], [1., 1., 1.]],
        BlockFace::West => [[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ymc_core::ChunkPos;

    #[test]
    fn single_block_has_six_faces() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        chunk.set(LocalPos::new(5, 5, 5), Block::Stone);
        let mesh = build_mesh(&chunk, &NoNeighbors);
        assert_eq!(mesh.opaque.quad_count(), 6);
        assert_eq!(mesh.opaque.vertices.len(), 24);
    }

    #[test]
    fn shared_faces_are_culled() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        chunk.set(LocalPos::new(5, 5, 5), Block::Stone);
        chunk.set(LocalPos::new(6, 5, 5), Block::Stone);
        let mesh = build_mesh(&chunk, &NoNeighbors);
        assert_eq!(mesh.opaque.quad_count(), 10);
    }

    #[test]
    fn water_goes_to_transparent_pass() {
        let mut chunk = Chunk::new(ChunkPos::new(0, 0));
        chunk.set(LocalPos::new(1, 1, 1), Block::Water);
        chunk.set(LocalPos::new(2, 1, 1), Block::Water);
        let mesh = build_mesh(&chunk, &NoNeighbors);
        assert_eq!(mesh.opaque.quad_count(), 0);
        assert_eq!(mesh.transparent.quad_count(), 10);
    }
}
