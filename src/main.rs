use bevy::prelude::*;
struct GamePlugin;
pub mod camera;
pub mod objects;
pub mod player;
pub mod ui;
pub mod world;

use crate::camera::camera::CameraPlugin;
use crate::objects::interactable::InteractionPlugin;
use crate::player::player::PlayerPlugin;
use crate::ui::cutscene::CutscenePlugin;
use crate::ui::dialogue::DialoguePlugin;
use crate::world::world::WorldPlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}
fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        GamePlugin,
        PlayerPlugin,
        InteractionPlugin,
        DialoguePlugin,
        CutscenePlugin,
        CameraPlugin,
        WorldPlugin,
    ));
    app.run();
}
