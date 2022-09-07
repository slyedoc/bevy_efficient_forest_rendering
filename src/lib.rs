use bevy::prelude::*;

pub mod chunk_grass;
pub mod chunk_instancing;

#[derive(Component, Debug)]
pub struct DistanceCulling {
    pub distance: f32,
}

impl Default for DistanceCulling {
    fn default() -> Self {
        Self { distance: 1000.0 }
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct Chunk {
    pub chunk_xy: [u32; 2],
}

