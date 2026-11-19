//! Headless preview: generates terrain around spawn, meshes it, and prints
//! a top-down map. The windowed `wgpu` renderer is the next milestone.

mod world;

use std::time::Instant;

use ymc_core::{Block, BlockPos, ChunkPos, CHUNK_SIZE};
use ymc_worldgen::{TerrainGenerator, WorldGenConfig};

use world::World;

const VIEW_RADIUS: i32 = 3;

fn main() {
    let seed = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| WorldGenConfig::default().seed);

    println!("YMeng Minecraft (Rust) v{}", env!("CARGO_PKG_VERSION"));
    println!("seed: {seed}\n");

    let config = WorldGenConfig {
        seed,
        ..Default::default()
    };
    let mut world = World::new(TerrainGenerator::new(config));
    let spawn = ChunkPos::new(0, 0);

    let start = Instant::now();
    world.load_area(spawn, VIEW_RADIUS);
    let gen_time = start.elapsed();

    let start = Instant::now();
    let mut opaque = 0;
    let mut transparent = 0;
    for pos in spawn.spiral(VIEW_RADIUS - 1) {
        let mesh = world.mesh_chunk(pos).expect("chunk loaded");
        opaque += mesh.opaque.quad_count();
        transparent += mesh.transparent.quad_count();
    }
    let mesh_time = start.elapsed();

    let meshed = (2 * VIEW_RADIUS - 1).pow(2);
    println!(
        "generated {} chunks in {:.1?}",
        world.chunk_count(),
        gen_time
    );
    println!("meshed {meshed} chunks in {mesh_time:.1?}");
    println!("  opaque quads:      {opaque}");
    println!("  transparent quads: {transparent}\n");

    print_map(&world, spawn, VIEW_RADIUS - 1);
}

fn print_map(world: &World, center: ChunkPos, radius: i32) {
    let size = CHUNK_SIZE as i32;
    let min = (center.x - radius) * size;
    let max = (center.x + radius + 1) * size;
    let sea = world.generator().config().sea_level;

    println!("top-down map (~ water, . sand, \" grass, T tree, ^ mountain)");
    for z in (min..max).step_by(2) {
        let row: String = (min..max).map(|x| map_char(world, x, z, sea)).collect();
        println!("{row}");
    }
}

fn map_char(world: &World, x: i32, z: i32, sea: usize) -> char {
    let height = world.generator().height(x, z);
    let top = (0..120)
        .rev()
        .filter_map(|y| world.block(BlockPos::new(x, y, z)))
        .find(|b| *b != Block::Air)
        .unwrap_or(Block::Air);
    match top {
        Block::Water => '~',
        Block::Sand => '.',
        Block::OakLeaves | Block::OakLog => 'T',
        Block::Grass if height > sea + 18 => '^',
        Block::Grass => '"',
        _ => '#',
    }
}
