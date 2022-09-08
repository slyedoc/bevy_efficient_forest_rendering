mod helper;

use helper::*;

use bevy::{
    asset::AssetServerSettings,
    math::prelude::*,
    math::Vec3A,
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        primitives::Aabb,
        render_resource::{AddressMode, FilterMode, PrimitiveTopology, SamplerDescriptor},
        texture::ImageSampler,
    },
    window::PresentMode,
};
use bevy_asset_loader::prelude::*;
use bevy_efficient_forest_rendering::{
    chunk_grass::{ChunkGrass, ChunkGrassBundle, GridConfig},
    chunk_instancing::{ChunkInstancing, ChunkInstancingBundle},
    DistanceCulling, ForestRenderingPlugin,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use iyes_loopless::prelude::*;
use std::f32::consts::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    AssetLoading,
    InGame,
}

#[derive(AssetCollection)]
pub struct FoliageAssets {
    // Note: using sub assets to load just want we need
    #[asset(path = "mushroom.glb#Mesh0/Primitive0")]
    mushroom_mesh: Handle<Mesh>,
    #[asset(path = "mushroom.glb#Texture0")]
    mushroom_texture: Handle<Image>,

    #[asset(path = "tree.glb#Mesh0/Primitive0")]
    tree_mesh: Handle<Mesh>,
    #[asset(path = "tree.glb#Texture0")]
    tree_texture: Handle<Image>,

    #[asset(path = "bush.glb#Mesh0/Primitive0")]
    bush_mesh: Handle<Mesh>,
    #[asset(path = "bush.glb#Texture0")]
    bush_texture: Handle<Image>,

    #[asset(path = "rock.glb#Mesh0/Primitive0")]
    rock_mesh: Handle<Mesh>,
    #[asset(path = "rock.glb#Texture0")]
    rock_texture: Handle<Image>,

    #[asset(path = "grass_ground_texture.png")]
    ground_texture: Handle<Image>,
}

pub struct GrassConfig {
    mesh: Handle<Mesh>,
    healthy_tip_color: Color, //Color::rgb(0.95, 0.91, 0.81),
    healthy_middle_color: Color,
    healthy_base_color: Color,

    unhealthy_tip_color: Color, //Should add favor ability map
    unhealthy_middle_color: Color,
    unhealthy_base_color: Color, //Color::rgb(0.22, 0.40, 0.255),
}

impl FromWorld for GrassConfig {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        Self {
            mesh: meshes.add(get_grass_straw_mesh()),
            healthy_tip_color: Color::rgb(0.66, 0.79 + 0.2, 0.34), //Color::rgb(0.95, 0.91, 0.81),
            healthy_middle_color: Color::rgb(0.40, 0.60, 0.3),
            healthy_base_color: Color::rgb(0.22, 0.40, 0.255),

            unhealthy_tip_color: Color::rgb(0.9, 0.95, 0.14), //Should add favorability map
            unhealthy_middle_color: Color::rgb(0.52, 0.57, 0.25),
            unhealthy_base_color: Color::rgb(0.22, 0.40, 0.255), //Color::rgb(0.22, 0.40, 0.255),
        }
    }
}

pub fn get_grass_straw_mesh() -> Mesh {
    let mut positions = Vec::with_capacity(5);
    let mut normals = Vec::with_capacity(5);
    let mut uvs = Vec::with_capacity(5);

    positions.push([0., 1.0, 0.0]);
    positions.push([0.05, 0.5, 0.0]);
    positions.push([-0.05, 0.5, 0.0]);
    positions.push([0.05, 0.0, 0.0]);
    positions.push([-0.05, 0.0, 0.0]);

    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);

    uvs.push([0.5, 1.0]);
    uvs.push([1.0, 0.5]);
    uvs.push([0.0, 0.5]);
    uvs.push([1.0, 0.0]);
    uvs.push([0.0, 0.0]);

    let indices = vec![0, 1, 2, 1, 3, 2, 2, 3, 4];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}

const NR_SIDE_CHUNKS: u32 = 30;
const INSTANCE_DENSITY: i32 = 1; //4
const CHUNK_SIZE: f32 = 30.;

fn main() {
    let mut app = App::new();

    app.add_loopless_state(GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::InGame)
                .with_collection::<FoliageAssets>(),
        )
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Fifo, // Don't cap at 60 fps
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.8)))
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(ForestRenderingPlugin)
        .init_resource::<GrassConfig>()
        .insert_resource(GridConfig {
            grid_center_xy: [0.0, 0.0],
            grid_half_extents: [
                NR_SIDE_CHUNKS as f32 * CHUNK_SIZE / 2.0,
                NR_SIDE_CHUNKS as f32 * CHUNK_SIZE / 2.0,
            ],
        })
        // shared helper plugin for examples
        .add_plugin(HelperPlugin)
        // Setup our scene
        .add_startup_system(spawn_camera_and_light) // add camera at startup
        .add_enter_system(GameState::InGame, spawn_ground)
        .add_enter_system(GameState::InGame, spawn_foliage);

    #[cfg(target_family = "wasm")]
    app.add_plugin(bevy_web_fullscreen::FullViewportPlugin);

    app.run();
}

fn spawn_camera_and_light(mut commands: Commands) {
    // Camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(FreeCameraController)
        //.insert(OrbitCamera::default()) // left this in
        .insert(Name::new("Camera"));

    // Sun
    commands
        .spawn_bundle(DirectionalLightBundle {
            transform: Transform {
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                ..default()
            },
            directional_light: DirectionalLight {
                illuminance: 30000.0,
                shadows_enabled: false,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Directional Light"));
}

fn spawn_ground(
    mut commands: Commands,
    foliage_assets: Res<FoliageAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    grid_config: Res<GridConfig>,
) {
    // Setup ground texture to repeat
    // TODO: should be cleaner way to do this
    let grass_image = images.get_mut(&foliage_assets.ground_texture).unwrap();
    grass_image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..default()
    });

    // setup ground mesh
    let mut ground_mesh = Mesh::from(shape::Plane {
        size: grid_config.get_size().x, // TODO: should it always be square
    });
    if let Some(VertexAttributeValues::Float32x2(uvs)) =
        ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    {
        for uv in uvs {
            uv[0] *= grid_config.grid_half_extents[0] / 4.0; //How dense texture should be sampled
            uv[1] *= grid_config.grid_half_extents[1] / 4.0;
        }
    }

    // Ground
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(ground_mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.34, 0.53, 0.255), //Adjust ground color
                base_color_texture: Some(foliage_assets.ground_texture.clone()),
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                metallic: 0.0,
                ..default()
            }),
            ..default()
        })
        .insert(Name::new("Ground"));
}

// Only here for ease of use to make refactoring easier while developing
struct Layer {
    mesh: Handle<Mesh>,
    image: Handle<Image>,
    transform: Transform,
    instance_count: u32,
    culling_distance: f32,
    name: &'static str,
}

fn spawn_foliage(
    mut commands: Commands,
    foliage_assets: Res<FoliageAssets>,
    grass_config: Res<GrassConfig>,
) {
    let nr_instances = (CHUNK_SIZE * CHUNK_SIZE * INSTANCE_DENSITY as f32) as u32;

    let mut tot_instances = 0;
    let mut tot_instances_grass = 0;

    let offset = CHUNK_SIZE * NR_SIDE_CHUNKS as f32 / 2.0;
    for chunk_x in 0..NR_SIDE_CHUNKS {
        for chunk_y in 0..NR_SIDE_CHUNKS {
            let chunk_half_height = 2.0;
            let chunk_x_pos = chunk_x as f32 * CHUNK_SIZE - offset;
            let chunk_y_pos = chunk_y as f32 * CHUNK_SIZE - offset;
            let chunk_aabb = Aabb {
                center: Vec3A::Y * chunk_half_height,
                half_extents: Vec3A::new(CHUNK_SIZE, chunk_half_height, CHUNK_SIZE), //Why do I need full chunk_size here?!, good question
            };

            commands
                .spawn_bundle((
                    Transform::from_xyz(
                        chunk_x as f32 * CHUNK_SIZE - offset,
                        0.0,
                        chunk_y as f32 * CHUNK_SIZE - offset,
                    ),
                    GlobalTransform::default(),
                    Visibility::default(),
                    ComputedVisibility::default(),
                    chunk_aabb.clone(),
                    Name::new(format!("Chunk {chunk_x}x{chunk_y}")),
                ))
                .with_children(|parent| {
                    for layer in vec![
                        Layer {
                            name: "Mushroom",
                            mesh: foliage_assets.mushroom_mesh.clone(),
                            image: foliage_assets.mushroom_texture.clone(),
                            transform: Transform {
                                scale: Vec3::splat(0.05),
                                ..default()
                            },
                            instance_count: nr_instances / 5,
                            culling_distance: 100.0,
                        },
                        Layer {
                            name: "Tree",
                            mesh: foliage_assets.tree_mesh.clone(),
                            image: foliage_assets.tree_texture.clone(),
                            transform: Transform {
                                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                                scale: Vec3::splat(0.2),
                                ..default()
                            },
                            instance_count: nr_instances / 15,
                            culling_distance: 200.0,
                        },
                        Layer {
                            name: "Bush",
                            mesh: foliage_assets.bush_mesh.clone(),
                            image: foliage_assets.bush_texture.clone(),
                            transform: Transform {
                                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                                scale: Vec3::splat(0.4),
                                ..default()
                            },
                            instance_count: nr_instances / 6,
                            culling_distance: 200.0,
                        },
                        Layer {
                            name: "Rock",
                            mesh: foliage_assets.rock_mesh.clone(),
                            image: foliage_assets.rock_texture.clone(),
                            transform: Transform {
                                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                                scale: Vec3::splat(0.6),
                                ..default()
                            },
                            instance_count: nr_instances / 10,
                            culling_distance: 200.0,
                        },
                    ] {
                        parent
                            .spawn_bundle(ChunkInstancingBundle {
                                mesh: layer.mesh.clone(),
                                chunk_instancing: ChunkInstancing::new(
                                    layer.instance_count,
                                    layer.image.clone(),
                                    layer.transform.clone(),
                                    CHUNK_SIZE,
                                ),
                                distance_culling: DistanceCulling {
                                    distance: layer.culling_distance,
                                },
                                aabb: chunk_aabb.clone(), // TODO: would like to avoid this all together and use parent AABB f
                                ..default()
                            })
                            .insert(Name::new(layer.name));
                        tot_instances += layer.instance_count;
                    }

                    // Grass
                    parent
                        .spawn_bundle(ChunkGrassBundle {
                            mesh: grass_config.mesh.clone(),
                            aabb: chunk_aabb.clone(),
                            chunk_grass: ChunkGrass {
                                time: 0.0,
                                healthy_tip_color: grass_config.healthy_tip_color,
                                healthy_middle_color: grass_config.healthy_middle_color,
                                healthy_base_color: grass_config.healthy_base_color,

                                unhealthy_tip_color: grass_config.unhealthy_tip_color,
                                unhealthy_middle_color: grass_config.unhealthy_middle_color,
                                unhealthy_base_color: grass_config.unhealthy_base_color,

                                chunk_xy: [chunk_x_pos, chunk_y_pos],
                                chunk_half_extents: [CHUNK_SIZE / 2.0, CHUNK_SIZE / 2.0],
                                nr_instances: nr_instances * 50,
                                growth_texture_id: 1,
                                scale: 1.6,
                                height_modifier: 0.6,
                            },
                            distance_culling: DistanceCulling { distance: 300.0 },
                            ..default()
                        })
                        .insert(Name::new(format!("Grass")));
                    tot_instances_grass += nr_instances * 50;
                });
        }
    }
    info!("Total instanced objects {:?}", tot_instances);
    info!("Total grass straws {:?}", tot_instances_grass);
}
