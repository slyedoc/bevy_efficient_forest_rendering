use std::f32::consts::*;

use bevy::{prelude::*, input::mouse::MouseMotion};


pub struct FreeCameraPlugin;

impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CameraControllerConfig>()
            .add_system(update_camera_controller);
    }
}

#[derive(Component)]
pub struct FreeCameraController;

pub struct CameraControllerConfig {
    pub enabled: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_look: MouseButton,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraControllerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.2,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_run: KeyCode::LShift,
            mouse_look: MouseButton::Right,
            walk_speed: 10.0,
            run_speed: 30.0,
            friction: 0.3,
            pitch: 0.0,
            yaw:  0.0,
            velocity: Vec3::ZERO,
        }
    }
}
fn update_camera_controller(
    time: Res<Time>,
    mut mouse_motion: EventReader<MouseMotion>,
    key_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<&mut Transform, With<FreeCameraController>>,
    mut config: ResMut<CameraControllerConfig>,
    mut windows: ResMut<Windows>,
) {
    let dt = time.delta_seconds();
    if let Some(window) = windows.get_primary_mut() {
        for mut transform in query.iter_mut() {
            if !config.enabled {
                continue;
            }
    
            // Handle key input
            let mut axis_input = Vec3::ZERO;
            if key_input.pressed(config.key_forward) {
                axis_input.z += 1.0;
            }
            if key_input.pressed(config.key_back) {
                axis_input.z -= 1.0;
            }
            if key_input.pressed(config.key_right) {
                axis_input.x += 1.0;
            }
            if key_input.pressed(config.key_left) {
                axis_input.x -= 1.0;
            }
            if key_input.pressed(config.key_up) {
                axis_input.y += 1.0;
            }
            if key_input.pressed(config.key_down) {
                axis_input.y -= 1.0;
            }
    
            // Apply movement update
            if axis_input != Vec3::ZERO {
                let max_speed = if key_input.pressed(config.key_run) {
                    config.run_speed
                } else {
                    config.walk_speed
                };
                config.velocity = axis_input.normalize() * max_speed;
            } else {
                let friction = config.friction.clamp(0.0, 1.0);
                config.velocity *= 1.0 - friction;
                if config.velocity.length_squared() < 1e-6 {
                    config.velocity = Vec3::ZERO;
                }
            }
            let right = transform.right();
            let forward = transform.forward();

            transform.translation += config.velocity.x * dt * right
                + config.velocity.y * dt * Vec3::Y
                + config.velocity.z * dt * forward;
    
            // Handle mouse look on mouse button
            let mut mouse_delta = Vec2::ZERO;
            if mouse_input.pressed(config.mouse_look) {
                #[cfg(not(target="wasm32"))]
                window.set_cursor_lock_mode(true);
                window.set_cursor_visibility(false);
            }
            if mouse_input.just_released(config.mouse_look) {
                #[cfg(not(target="wasm32"))]
                window.set_cursor_lock_mode(false);
                window.set_cursor_visibility(true);
            }
            if mouse_input.pressed(config.mouse_look) {
                for mouse_event in mouse_motion.iter() {
                    mouse_delta += mouse_event.delta;
                }
            }
    
            if mouse_delta != Vec2::ZERO {                
                let (mut yaw, mut pitch, _roll) =  transform.rotation.to_euler(EulerRot::YXZ);
                yaw -= mouse_delta.x * config.sensitivity * time.delta_seconds();
                pitch -= mouse_delta.y * config.sensitivity * time.delta_seconds();

                let pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
                transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)
                                
            }
        }
    }
}