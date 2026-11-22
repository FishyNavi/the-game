use bevy::{input::keyboard::KeyCode::KeyA, prelude::*};
use std::{default, f32::consts::FRAC_1_SQRT_2, time::Duration};

use crate::ui::cutscene::Cutscene;
use crate::ui::dialogue::Dialogue;

const MAX_SPEED: f32 = 300.;
const ACCELERATION: f32 = 1500.;
const FRICTION: f32 = 1000.;

const GRAVITY: f32 = 1500.;
const TERMINAL_VELOCITY: f32 = 600.;
const JUMP_SPEED: f32 = 650.;
const COYOTE_TIME: f32 = 0.1;
const JUMP_BUFFER: f32 = 0.1;

const PLAYER_SIZE: Vec2 = Vec2::new(49., 64.);

#[derive(Component, Default)]
pub struct Player {
    on_ground: bool,
    wall_timer: f32,
    touching_wall: bool,
    wall_dir: f32,
    coyote_timer: f32,
    jump_buffer: f32,
}

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Collider(pub Vec2);

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (handle_player_input, apply_gravity, move_and_collide).chain(),
        );
        app.add_systems(
            Update,
            (
                update_player_animation,
                on_player_anim_state_changed,
                update_animations,
            )
                .chain()
                .after(move_and_collide),
        );
    }
}

fn spawn_solid(commands: &mut Commands, pos: Vec2, size: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.5, 0.4, 0.5), size),
        Transform::from_translation(pos.extend(0.)),
        Collider(size),
    ));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("sprites/sprite.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 5, None, None);
    let layout = texture_atlas_layouts.add(layout);
    let animation_config = AnimationConfig::new(1, 8, 8, true);
    let index = animation_config.first;
    // commands.spawn((
    //     Sprite::from_color(Color::srgb(0.7, 0.5, 0.7), PLAYER_SIZE),
    //     Transform::from_xyz(0., 0., 1.),
    //     Player::default(),
    //     Velocity::default(),
    // ));
    let state = PlayerAnimState::Idle;
    let config = state.config();

    commands.spawn((
        Sprite {
            image: texture.clone(),
            texture_atlas: Some(TextureAtlas { layout, index }),
            ..default()
        },
        state,
        config,
        Transform::from_xyz(0., 0., 1.).with_scale(Vec3::new(2.0, 2.0, 1.0)),
        Player::default(),
        Velocity::default(),
    ));
    // spawn_solid(&mut commands, Vec2::new(0., -200.), Vec2::new(1400., 40.));
    // spawn_solid(&mut commands, Vec2::new(-250., -80.), Vec2::new(160., 20.));
    // spawn_solid(&mut commands, Vec2::new(0., 0.), Vec2::new(20., 320.));
}

fn move_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_delta {
        target
    } else {
        current + diff.signum() * max_delta
    }
}
fn update_player_animation(
    input: Res<ButtonInput<KeyCode>>,
    query: Single<(&Velocity, &Player, &mut Sprite, &mut PlayerAnimState)>,
) {
    let (vel, player, mut sprite, mut state) = query.into_inner();
    if vel.0.x < -0.1 {
        sprite.flip_x = true;
    } else if vel.0.x > 0.1 {
        sprite.flip_x = false;
    }
    let next_state = if !player.on_ground
        && player.touching_wall
        && vel.0.y < 0.
        && input.any_pressed([KeyCode::KeyD, KeyCode::KeyA])
    {
        PlayerAnimState::SlidingWall
    } else if !player.on_ground {
        PlayerAnimState::Falling
    } else if vel.0.x.abs() > 1. {
        PlayerAnimState::Run
    } else {
        PlayerAnimState::Idle
    };
    state.set_if_neq(next_state);
}
pub fn handle_player_input(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player: Single<(&mut Velocity, &mut Player)>,
    dialogue: Res<Dialogue>,
    cutscene: Res<Cutscene>,
) {
    let (mut vel, mut player) = player.into_inner();
    let dt = time.delta_secs();

    let locked = dialogue.is_active() || cutscene.active();

    let mut h = 0.;
    if !locked {
        if input.pressed(KeyCode::KeyA) {
            h -= 1.;
        }
        if input.pressed(KeyCode::KeyD) {
            h += 1.;
        }
    }

    let target = h * MAX_SPEED;
    let x = vel.0.x;

    let rate = if h == 0. { FRICTION } else { ACCELERATION };

    vel.0.x = move_toward(x, target, rate * dt);

    if player.on_ground {
        player.coyote_timer = COYOTE_TIME;
    } else {
        player.coyote_timer = (player.coyote_timer - dt).max(0.);
    }

    if input.just_pressed(KeyCode::Space) && !locked {
        player.jump_buffer = JUMP_BUFFER;
    } else {
        player.jump_buffer = (player.jump_buffer - dt).max(0.);
    }

    player.wall_timer -= dt;

    if !input.any_pressed([KeyCode::KeyD, KeyCode::KeyA]) {
        player.touching_wall = false;
    } else {
        player.touching_wall = player.wall_timer > 0.; ////// CHANGE
    }
    if player.jump_buffer > 0. {
        if player.coyote_timer > 0. {
            vel.0.y = JUMP_SPEED;
            player.jump_buffer = 0.;
            player.coyote_timer = 0.;
            player.on_ground = false;
        } else if player.wall_timer > 0. {
            vel.0.y = JUMP_SPEED * FRAC_1_SQRT_2;
            vel.0.x = JUMP_SPEED * FRAC_1_SQRT_2 * -player.wall_dir;
            player.wall_timer = 0.;
        }
    }
}

fn apply_gravity(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player_query: Single<(&mut Velocity, &Player)>,
) {
    let (mut vel, player) = player_query.into_inner();
    let mult = if vel.0.y < 0. {
        1.2
    } else if vel.0.y > 0. && !input.pressed(KeyCode::Space) {
        2.5
    } else {
        1.
    };
    let tvel = if player.touching_wall {
        -TERMINAL_VELOCITY / 4.
    } else {
        -TERMINAL_VELOCITY
    };
    vel.0.y = (vel.0.y - GRAVITY * mult * time.delta_secs().min(1. / 60.)).max(tvel);
}

fn box_overlaps(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    const EPS: f32 = 0.001;
    let min_dist = (a_size + b_size) * 0.5 - EPS;
    let delta = (a_pos - b_pos).abs();

    delta.cmplt(min_dist).all()
}

pub fn move_and_collide(
    time: Res<Time>,
    player: Single<(&mut Velocity, &mut Player, &mut Transform)>,
    solids: Query<(&Transform, &Collider), Without<Player>>,
) {
    let (mut vel, mut player, mut tf) = player.into_inner();
    let dt = time.delta_secs();

    player.touching_wall = false;
    tf.translation.x += vel.0.x * dt;
    for (stf, col) in &solids {
        let spos = stf.translation.truncate();
        if box_overlaps(tf.translation.truncate(), PLAYER_SIZE, spos, col.0) {
            let dir = vel.0.x.signum();
            if vel.0.x > 0. {
                tf.translation.x = spos.x - col.0.x / 2. - PLAYER_SIZE.x / 2.;
            } else if vel.0.x < 0. {
                tf.translation.x = spos.x + col.0.x / 2. + PLAYER_SIZE.x / 2.;
            }
            if vel.0.x != 0. {
                player.touching_wall = true;
                player.wall_dir = dir;
                player.wall_timer = 0.2;
            }
            vel.0.x = 0.;
        }
    }
    tf.translation.y += vel.0.y * dt;
    player.on_ground = false;
    for (stf, col) in &solids {
        let spos = stf.translation.truncate();
        if box_overlaps(tf.translation.truncate(), PLAYER_SIZE, spos, col.0) {
            if vel.0.y < 0. {
                tf.translation.y = spos.y + col.0.y / 2. + PLAYER_SIZE.y / 2.;
                player.on_ground = true;
            } else if vel.0.y > 0. {
                tf.translation.y = spos.y - col.0.y / 2. - PLAYER_SIZE.y / 2.;
            }
            vel.0.y = 0.;
        }
    }
}

#[derive(Component)]
struct AnimationConfig {
    first: usize,
    last: usize,
    timer: Timer,
    looping: bool,
}

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8, looping: bool) -> Self {
        Self {
            first: first,
            last: last,
            timer: Self::timer_from_fps(fps),
            looping,
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(
            Duration::from_secs_f32(1.0 / (fps as f32)),
            TimerMode::Repeating,
        )
    }
}
#[derive(Component, PartialEq, Eq, Clone, Copy, Debug, Default)]
enum PlayerAnimState {
    #[default]
    Idle,
    Run,
    Falling,
    SlidingWall,
}
impl PlayerAnimState {
    fn config(&self) -> AnimationConfig {
        match self {
            PlayerAnimState::Idle => AnimationConfig::new(0, 7, 8, true),
            PlayerAnimState::Run => AnimationConfig::new(8, 15, 12, true),
            PlayerAnimState::Falling => AnimationConfig::new(16, 23, 8, true),
            PlayerAnimState::SlidingWall => AnimationConfig::new(32, 32, 1, true),
        }
    }
}
fn on_player_anim_state_changed(
    query: Single<(&PlayerAnimState, &mut AnimationConfig, &mut Sprite), Changed<PlayerAnimState>>,
) {
    let (state, mut config, mut sprite) = query.into_inner();
    *config = state.config();
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = config.first
    }
}

fn update_animations(time: Res<Time>, query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut config, mut sprite) in query {
        config.timer.tick(time.delta());
        if config.timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if atlas.index == config.last {
                if config.looping {
                    atlas.index = config.first
                }
            } else {
                atlas.index += 1;
            }
        }
    }
}
