#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

pub mod chunk_grass;
pub mod chunk_instancing;

pub struct ForestRenderingPlugin;

impl Plugin for ForestRenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(chunk_grass::ChunkGrassPlugin)
            .add_plugin(chunk_instancing::ChunkInstancingPlugin)
            .register_inspectable::<DistanceCulling>();
    }
}

#[derive(Component, Inspectable, Debug)]
pub struct DistanceCulling {
    pub distance: f32,
}

impl Default for DistanceCulling {
    fn default() -> Self {
        Self { distance: 1000.0 }
    }
}