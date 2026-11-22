use bevy::prelude::*;
struct GamePlugin;
pub mod player;

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
    app.add_plugins((DefaultPlugins, GamePlugin, player::player::PlayerPlugin));
    app.run();
}
