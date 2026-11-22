use bevy::prelude::*;
use std::f32::consts::FRAC_1_SQRT_2;

const MAX_SPEED: f32 = 300.;
const ACCELERATION: f32 = 2000.;
const FRICTION: f32 = 1000.;

const GRAVITY: f32 = 1500.;
const TERMINAL_VELOCITY: f32 = 600.;
const JUMP_SPEED: f32 = 650.;
const COYOTE_TIME: f32 = 0.1;
const JUMP_BUFFER: f32 = 0.1;

const PLAYER_SIZE: Vec2 = Vec2::new(24., 32.);

#[derive(Component, Default)]
pub struct Player {
    on_ground: bool,
    wall_timer: f32,
    wall_on: f32,
    coyote_timer: f32,
    jump_buffer: f32,
}

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider(Vec2);

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (handle_player_input, apply_gravity, move_and_collide).chain(),
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

fn setup(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.7, 0.5, 0.7), PLAYER_SIZE),
        Transform::from_xyz(0., 0., 1.),
        Player::default(),
        Velocity::default(),
    ));
    spawn_solid(&mut commands, Vec2::new(0., -200.), Vec2::new(1400., 40.));
    spawn_solid(&mut commands, Vec2::new(-250., -80.), Vec2::new(160., 20.));
    spawn_solid(&mut commands, Vec2::new(0., 0.), Vec2::new(20., 320.));
}

fn move_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_delta {
        target
    } else {
        current + diff.signum() * max_delta
    }
}

fn handle_player_input(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player: Single<(&mut Velocity, &mut Player)>,
) {
    let (mut vel, mut player) = player.into_inner();
    let dt = time.delta_secs();

    let mut h = 0.;
    if input.pressed(KeyCode::KeyA) {
        h -= 1.;
    }
    if input.pressed(KeyCode::KeyD) {
        h += 1.;
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

    if input.just_pressed(KeyCode::Space) {
        player.jump_buffer = JUMP_BUFFER;
    } else {
        player.jump_buffer = (player.jump_buffer - dt).max(0.);
    }
    player.wall_timer -= dt;

    if player.jump_buffer > 0. {
        if player.coyote_timer > 0. {
            vel.0.y = JUMP_SPEED;
            player.jump_buffer = 0.;
            player.coyote_timer = 0.;
            player.on_ground = false;
        } else if player.wall_timer > 0. {
            vel.0.y = JUMP_SPEED * FRAC_1_SQRT_2;
            vel.0.x = JUMP_SPEED * FRAC_1_SQRT_2 * -player.wall_on;
            player.jump_buffer = 0.;
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
    let tvel = if player.wall_timer > 0. {
        -TERMINAL_VELOCITY / 3.
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

fn move_and_collide(
    time: Res<Time>,
    player: Single<(&mut Velocity, &mut Player, &mut Transform)>,
    solids: Query<(&Transform, &Collider), Without<Player>>,
) {
    let (mut vel, mut player, mut tf) = player.into_inner();
    let dt = time.delta_secs();
    tf.translation.x += vel.0.x * dt;
    for (stf, col) in &solids {
        let spos = stf.translation.truncate();
        if box_overlaps(tf.translation.truncate(), PLAYER_SIZE, spos, col.0) {
            player.wall_timer = 0.1;
            if vel.0.x > 0. {
                player.wall_on = 1.;
                tf.translation.x = spos.x - col.0.x / 2. - PLAYER_SIZE.x / 2.;
            } else if vel.0.x < 0. {
                player.wall_on = -1.;
                tf.translation.x = spos.x + col.0.x / 2. + PLAYER_SIZE.x / 2.;
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


