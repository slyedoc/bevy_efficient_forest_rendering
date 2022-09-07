mod overlay;
use bevy::prelude::*;

use overlay::OverlayPlugin;



pub struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(OverlayPlugin);
    }
}
