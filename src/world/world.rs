use bevy::prelude::*;

use crate::{
    objects::interactable::{CutsceneZone, Interactable},
    player::player::Collider,
    ui::{
        cutscene::{CutsceneActor, StartCutscene, Step},
        dialogue::{Line, Talk},
    },
};

const PLAYER_FACE: &str = "portraits/player.png";
const AUTHOR_FACE: &str = "portraits/author.png";
const NPC_FACE: &str = "portraits/box.png";

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.53, 0.75, 0.9)))
            .add_systems(Startup, (spawn_world, start_intro));
    }
}

fn solid(commands: &mut Commands, pos: Vec2, size: Vec2, color: Color) {
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_translation(pos.extend(0.)),
        Collider(size),
    ));
}

fn spawn_world(mut commands: Commands) {
    let ground = Color::srgb(0.30, 0.55, 0.25);
    let stone = Color::srgb(0.45, 0.42, 0.40);
    solid(
        &mut commands,
        Vec2::new(400., -200.),
        Vec2::new(2400., 40.),
        ground,
    );
    solid(
        &mut commands,
        Vec2::new(-815., 0.),
        Vec2::new(30., 400.),
        stone,
    );
    solid(
        &mut commands,
        Vec2::new(1615., 0.),
        Vec2::new(30., 400.),
        stone,
    );
    solid(
        &mut commands,
        Vec2::new(300., -160.),
        Vec2::new(100., 40.),
        stone,
    );
    solid(
        &mut commands,
        Vec2::new(520., -90.),
        Vec2::new(160., 20.),
        stone,
    );
    solid(
        &mut commands,
        Vec2::new(760., -10.),
        Vec2::new(160., 20.),
        stone,
    );
    solid(
        &mut commands,
        Vec2::new(980., -100.),
        Vec2::new(30., 160.),
        stone,
    );
    solid(
        &mut commands,
        Vec2::new(1350., -160.),
        Vec2::new(500., 40.),
        ground,
    );
    commands.spawn((
        CutsceneActor("author"),
        Sprite::from_color(Color::srgb(0.9, 0.8, 0.2), Vec2::new(32., 48.)),
        Transform::from_xyz(-250., -156., 0.),
        Interactable { range: 70. },
        Talk::from_speaker(
            "Game author",
            AUTHOR_FACE,
            &[
                "Hey, you. You're finally awake.",
                "A and D to move, Space to jump. ",
                "You can also wall jump",
                "As you can see stuff is not really finished up here, but it will change, i promise",
            ],
        ),
    ));

    commands.spawn((
        CutsceneActor("npc"),
        Sprite::from_color(Color::srgb(0.7, 0.4, 0.8), Vec2::new(32., 56.)),
        Transform::from_xyz(1300., -112., 0.),
    ));

    commands.spawn((
        Transform::from_xyz(1180., -100., 0.),
        CutsceneZone {
            size: Vec2::new(80., 240.),
            steps: vec![
                Step::Say(vec![
                    Line::line("NPC", NPC_FACE, "You're very good at platformers, wow"),
                    Line::line(
                        "NPC",
                        NPC_FACE,
                        "Come, there is something I need to show you.",
                    ),
                ]),
                Step::Walk {
                    actor: "npc",
                    x: 1500.,
                    speed: 100.,
                },
                Step::Wait(0.5),
                Step::Say(vec![Line::line("NPC", NPC_FACE, "Follow me")]),
                Step::Run(spawn_zone),
            ],
        },
    ));
}
fn spawn_zone(commands: &mut Commands) {
    commands.spawn((
        Transform::from_translation((Vec2::new(1180., -100.) + Vec2::new(230., 12.)).extend(0.)),
        CutsceneZone {
            size: Vec2::new(80., 240.),
            steps: vec![
                Step::Fade { to: 1., secs: 1.0 },
                Step::ShowImage("cutscenes/end.png"),
                Step::Fade { to: 0., secs: 1.0 },
                Step::Wait(1.5),
                Step::Say(vec![Line::line(
                    "npc",
                    NPC_FACE,
                    "There's nothing more to show!",
                )]),
                Step::Fade { to: 1., secs: 1.0 },
                Step::HideImage,
                Step::Say(vec![Line::line(
                    "Player",
                    PLAYER_FACE,
                    "To be continued i guess",
                )]),
                Step::Fade { to: 0., secs: 1.0 },
            ],
        },
    ));
}
fn start_intro(mut start: MessageWriter<StartCutscene>) {
    start.write(StartCutscene(vec![
        Step::Fade { to: 1., secs: 0.01 },
        Step::ShowImage("cutscenes/intro.png"),
        Step::Fade { to: 0., secs: 1.0 },
        Step::Wait(1.5),
        Step::Say(vec![Line::line("Player", PLAYER_FACE, "Bark")]),
        Step::Say(vec![Line::line("Player", PLAYER_FACE, "Where... am I?")]),
        Step::Fade { to: 1., secs: 1.0 },
        Step::HideImage,
        Step::Fade { to: 0., secs: 1.0 },
        Step::Say(vec![Line::line(
            "Player",
            PLAYER_FACE,
            "I should look around. Maybe someone nearby can help.",
        )]),
    ]));
}
