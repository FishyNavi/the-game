use bevy::prelude::*;

use crate::{
    player::player::{Player, move_and_collide},
    ui::cutscene::Cutscene,
};

const FOLLOW_SPEED: f32 = 8.;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            follow_player
                .after(move_and_collide)
                .run_if(not_in_cutscene),
        );
    }
}

fn not_in_cutscene(cutscene: Res<Cutscene>) -> bool {
    !cutscene.active()
}

fn follow_player(
    time: Res<Time>,
    player: Single<&Transform, With<Player>>,
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let target = player.translation.truncate();
    let current = camera.translation.truncate();

    let t = 1. - (-FOLLOW_SPEED * time.delta_secs()).exp();
    let next = current.lerp(target, t);

    camera.translation.x = next.x;
    camera.translation.y = next.y;
}
