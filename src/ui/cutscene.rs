use crate::{
    player::player::Velocity,
    ui::dialogue::{Dialogue, DialogueSet, Line, StartDialogue},
};
use bevy::{color::Alpha, prelude::*};
use std::collections::VecDeque;
#[derive(Component)]
struct CutsceneImage;
#[derive(Component)]
pub struct CutsceneActor(pub &'static str);

#[derive(Clone)]
pub enum Step {
    Wait(f32),
    Say(Vec<Line>),
    Walk {
        actor: &'static str,
        x: f32,
        speed: f32,
    },
    Pan {
        to: Vec2,
        speed: f32,
    },
    Fade {
        to: f32,
        secs: f32,
    },
    Run(fn(&mut Commands)),
    ShowImage(&'static str),
    HideImage,
}

#[derive(Message)]
pub struct StartCutscene(pub Vec<Step>);

#[derive(Resource, Default)]
pub struct Cutscene {
    steps: VecDeque<Step>,
    fresh: bool,
    t: f32,
    from: f32,
}
impl Cutscene {
    pub fn active(&self) -> bool {
        !self.steps.is_empty()
    }
}

#[derive(Component)]
struct FadeOverlay;

pub struct CutscenePlugin;
impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cutscene>()
            .add_message::<StartCutscene>()
            .add_systems(Startup, setup_overlays)
            .add_systems(
                Update,
                run_cutscene
                    .after(crate::player::player::handle_player_input)
                    .before(DialogueSet),
            );
    }
}

fn setup_overlays(mut commands: Commands) {
    commands.spawn((
        FadeOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        BackgroundColor(Color::srgba(0., 0., 0., 0.)),
        GlobalZIndex(50),
    ));
    commands.spawn((
        CutsceneImage,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ImageNode::default(),
        Visibility::Hidden,
        GlobalZIndex(40),
    ));
}
fn run_cutscene(
    mut cs: ResMut<Cutscene>,
    mut starts: MessageReader<StartCutscene>,
    time: Res<Time>,
    dialogue: Res<Dialogue>,
    mut say: MessageWriter<StartDialogue>,
    mut commands: Commands,
    mut actors: Query<(&CutsceneActor, &mut Transform, Option<&mut Velocity>)>,
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<CutsceneActor>)>,
    mut fade: Single<&mut BackgroundColor, With<FadeOverlay>>,
    assets: Res<AssetServer>,
    mut image: Single<(&mut ImageNode, &mut Visibility), With<CutsceneImage>>,
) {
    for StartCutscene(steps) in starts.read() {
        if !cs.active() {
            cs.steps = steps.iter().cloned().collect();
            cs.fresh = true;
            cs.t = 0.;
        }
    }

    let Some(step) = cs.steps.front().cloned() else {
        return;
    };
    let dt = time.delta_secs().min(1. / 30.);
    let first = cs.fresh;
    cs.fresh = false;

    let done = match step {
        Step::Wait(secs) => {
            cs.t += dt;
            cs.t >= secs
        }
        Step::Say(lines) => {
            if first {
                say.write(StartDialogue(lines));
                false
            } else {
                !dialogue.is_active()
            }
        }
        Step::Walk { actor, x, speed } => {
            let mut done = true;
            for (a, mut tf, vel) in &mut actors {
                if a.0 != actor {
                    continue;
                }
                let dx = x - tf.translation.x;
                let stride = speed * dt;
                if dx.abs() <= stride {
                    tf.translation.x = x;
                    if let Some(mut v) = vel {
                        v.0.x = 0.;
                    }
                } else {
                    done = false;
                    let dir = dx.signum();
                    match vel {
                        Some(mut v) => v.0.x = dir * speed,
                        None => tf.translation.x += dir * stride,
                    }
                }
            }
            done
        }
        Step::Pan { to, speed } => {
            let d = to - camera.translation.truncate();
            let stride = speed * dt;
            if d.length() <= stride {
                camera.translation.x = to.x;
                camera.translation.y = to.y;
                true
            } else {
                camera.translation += (d.normalize() * stride).extend(0.);
                false
            }
        }
        Step::Fade { to, secs } => {
            if first {
                cs.from = fade.0.alpha();
                cs.t = 0.;
            }
            cs.t += dt;
            let k = (cs.t / secs.max(0.001)).min(1.);
            fade.0.set_alpha(cs.from + (to - cs.from) * k);
            k >= 1.
        }
        Step::Run(f) => {
            f(&mut commands);
            true
        }
        Step::ShowImage(path) => {
            let (img, vis) = &mut *image;
            img.image = assets.load(path);
            **vis = Visibility::Inherited;
            true
        }
        Step::HideImage => {
            let (_, vis) = &mut *image;
            **vis = Visibility::Hidden;
            true
        }
    };

    if done {
        cs.steps.pop_front();
        cs.fresh = true;
        cs.t = 0.;
    }
}
