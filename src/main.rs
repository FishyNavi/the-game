use bevy::prelude::*;
struct GamePlugin;
pub mod objects;
pub mod player;
pub mod ui;
use crate::objects::interactables::InteractionPlugin;
use crate::player::player::PlayerPlugin;
use crate::ui::dialogue::DialoguePlugin;
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
    ));
    app.run();
}
