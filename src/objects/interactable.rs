use crate::{
    player::player::Player,
    ui::cutscene::{Cutscene, CutsceneActor, StartCutscene, Step},
    ui::dialogue::{Dialogue, DialogueSet, Line, StartDialogue, Talk},
};

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

#[derive(Component)]
pub struct CutsceneTrigger(pub Vec<Step>);

#[derive(Component)]
pub struct CutsceneZone {
    pub size: Vec2,
    pub steps: Vec<Step>,
}

#[derive(Component)]
pub struct DialogueZone {
    pub size: Vec2,
    pub lines: Vec<Line>,
}

#[derive(Resource, Default)]
struct Nearby(Option<Entity>);

#[derive(Component)]
struct Prompt;

#[derive(Component)]
pub struct Interactable {
    pub range: f32,
}
fn idle(dialogue: Res<Dialogue>, cutscene: Res<Cutscene>) -> bool {
    !dialogue.is_active() && !cutscene.active()
}

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Nearby>()
            .add_systems(Startup, (setup, spawn_stuff))
            .add_systems(
                Update,
                (
                    find_nearby,
                    interact.run_if(input_just_pressed(KeyCode::KeyE)),
                    dialogue_zones.run_if(idle),
                    cutscene_zones.run_if(idle),
                    update_prompt,
                )
                    .chain()
                    .before(DialogueSet),
            );
    }
}
fn setup(mut commands: Commands) {
    commands.spawn((
        Prompt,
        Text2d::new("E"),
        TextFont {
            font_size: FontSize::Px(18.),
            ..default()
        },
        Transform::default(),
        Visibility::Hidden,
    ));
}

fn spawn_stuff(mut commands: Commands) {
    //     commands.spawn((
    //         CutsceneActor("Box"),
    //         Sprite::from_color(Color::srgb(1., 1., 1.), Vec2::splat(48.)),
    //         Transform::from_xyz(-250., -156., 0.),
    //         Interactable { range: 70. },
    //         Talk::from_speaker(
    //             "Box",
    //             "portraits/box.png",
    //             &[
    //                 "...",
    //                 "................",
    //                 ".........",
    //                 "........................................",
    //             ],
    //         ),
    //     ));
    commands.spawn((
        Transform::from_xyz(250., -60., 0.),
        DialogueZone {
            size: Vec2::new(64., 128.),
            lines: vec![Line::line(
                "Player",
                "portraits/player.png",
                "Hmmm, i don't have a name.",
            )],
        },
    ));
    //     commands.spawn((
    //         Transform::from_xyz(-250., -150., 0.),
    //         CutsceneZone {
    //             size: Vec2::new(64., 128.),
    //             steps: vec![
    //                 Step::Fade { to: 1., secs: 0.5 },
    //                 Step::Pan {
    //                     to: Vec2::new(700., 0.),
    //                     speed: 600.,
    //                 },
    //                 Step::Fade { to: 0., secs: 0.5 },
    //                 Step::Say(vec![Line::line("Box", "portraits/box.png", "...")]),
    //                 Step::Walk {
    //                     actor: "Box",
    //                     x: 650.,
    //                     speed: 120.,
    //                 },
    //                 Step::Wait(0.5),
    //                 Step::Pan {
    //                     to: Vec2::ZERO,
    //                     speed: 600.,
    //                 },
    //             ],
    //         },
    //     ));
}

fn find_nearby(
    dialogue: Res<Dialogue>,
    mut nearby: ResMut<Nearby>,
    player: Single<&Transform, With<Player>>,
    interactables: Query<(Entity, &Transform, &Interactable), Without<Player>>,
) {
    if dialogue.is_active() {
        nearby.0 = None;
        return;
    }
    let ptf = player.translation.truncate();
    nearby.0 = interactables
        .iter()
        .map(|(e, tf, i)| (e, tf.translation.truncate().distance(ptf), i.range))
        .filter(|(_, dist, range)| dist <= range)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(e, ..)| e)
}

fn interact(
    nearby: Res<Nearby>,
    talkers: Query<&Talk>,
    mut start: MessageWriter<StartDialogue>,
    ctriggers: Query<&CutsceneTrigger>,
    mut cutscenes: MessageWriter<StartCutscene>,
) {
    let Some(e) = nearby.0 else { return };
    if let Ok(talk) = talkers.get(e) {
        start.write(StartDialogue(talk.0.clone()));
    }
    if let Ok(t) = ctriggers.get(e) {
        cutscenes.write(StartCutscene(t.0.clone()));
    }
}

fn dialogue_zones(
    mut commands: Commands,
    dialogue: Res<Dialogue>,
    player: Single<&Transform, With<Player>>,
    zones: Query<(Entity, &Transform, &DialogueZone), Without<Player>>,
    mut start: MessageWriter<StartDialogue>,
) {
    if dialogue.is_active() {
        return;
    }
    let p = player.translation.truncate();
    for (e, tf, zone) in &zones {
        let d = (p - tf.translation.truncate()).abs();
        if d.x < zone.size.x / 2. && d.y < zone.size.y / 2. {
            start.write(StartDialogue(zone.lines.clone()));
            commands.entity(e).despawn();
        }
    }
}
fn cutscene_zones(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    zones: Query<(Entity, &Transform, &CutsceneZone), Without<Player>>,
    mut start: MessageWriter<StartCutscene>,
) {
    let p = player.translation.truncate();
    for (e, tf, zone) in &zones {
        let d = (p - tf.translation.truncate()).abs();
        if d.x < zone.size.x / 2. && d.y < zone.size.y / 2. {
            info!("cutscene zone triggered");
            start.write(StartCutscene(zone.steps.clone()));
            commands.entity(e).despawn();
            break;
        }
    }
}

fn update_prompt(
    nearby: Res<Nearby>,
    targets: Query<&Transform, Without<Prompt>>,
    prompt: Single<(&mut Transform, &mut Visibility), With<Prompt>>,
) {
    let (mut tf, mut vis) = prompt.into_inner();
    match nearby.0.and_then(|e| targets.get(e).ok()) {
        Some(target) => {
            tf.translation = target.translation + Vec3::new(0., 40., 5.);
            *vis = Visibility::Inherited;
        }
        None => *vis = Visibility::Hidden,
    }
}
