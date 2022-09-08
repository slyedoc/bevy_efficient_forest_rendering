mod overlay;
mod free_camera;
mod orbit_camera;

use bevy::prelude::*;

use overlay::OverlayPlugin;
pub use free_camera::*;
pub use orbit_camera::*;

pub struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(OverlayPlugin)
        .add_plugin(FreeCameraPlugin)
        .add_plugin(OrbitCameraPlugin);
    }
}
