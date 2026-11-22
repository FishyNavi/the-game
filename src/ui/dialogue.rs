use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

#[derive(Clone)]
pub struct Line {
    pub speaker: String,
    pub face: Option<String>,
    pub text: String,
}
impl Line {
    pub fn line(speaker: &str, face: &str, text: &str) -> Self {
        Self {
            speaker: speaker.to_string(),
            face: Some(face.to_string()),
            text: text.to_string(),
        }
    }
}

#[derive(Component)]
pub struct Talk(pub Vec<Line>);
impl Talk {
    pub fn from_speaker(speaker: &str, face: &str, lines: &[&str]) -> Self {
        Self(lines.iter().map(|t| Line::line(speaker, face, t)).collect())
    }
}

#[derive(Message)]
pub struct StartDialogue(pub Vec<Line>);

#[derive(Resource, Default)]
pub struct Dialogue {
    lines: Vec<Line>,
    index: usize,
}
impl Dialogue {
    pub fn is_active(&self) -> bool {
        !self.lines.is_empty()
    }
}

#[derive(Component)]
struct DialogueBox;
#[derive(Component)]
struct DialogueFace;
#[derive(Component)]
struct DialogueName;
#[derive(Component)]
struct DialogueText;

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub struct DialogueSet;

pub struct DialoguePlugin;
impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Dialogue>()
            .add_message::<StartDialogue>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    advance_dialogue.run_if(input_just_pressed(KeyCode::KeyE)),
                    begin_dialogue,
                    update_dialogue_ui,
                )
                    .chain()
                    .in_set(DialogueSet),
            );
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        DialogueBox,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.),
            left: Val::Percent(10.),
            width: Val::Percent(80.),
            padding: UiRect::all(Val::Px(16.)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0., 0.1, 0.9)),
        GlobalZIndex(100),
        Visibility::Hidden,
        children![
            (
                DialogueFace,
                Node {
                    width: Val::Px(128.),
                    height: Val::Px(128.),
                    flex_shrink: 0.,

                    ..default()
                },
                ImageNode::default(),
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.),
                    flex_grow: 1.,
                    ..default()
                },
                children![
                    (
                        DialogueName,
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(22.),
                            ..default()
                        },
                        TextColor(Color::srgb(1., 0.85, 0.95)),
                    ),
                    (
                        DialogueText,
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(18.),
                            ..default()
                        }
                    )
                ]
            )
        ],
    ));
}

fn advance_dialogue(mut dialogue: ResMut<Dialogue>) {
    if !dialogue.is_active() {
        return;
    }
    dialogue.index += 1;
    if dialogue.index >= dialogue.lines.len() {
        dialogue.lines.clear();
        dialogue.index = 0;
    }
}

fn begin_dialogue(mut reader: MessageReader<StartDialogue>, mut dialogue: ResMut<Dialogue>) {
    for StartDialogue(lines) in reader.read() {
        if !dialogue.is_active() && !lines.is_empty() {
            dialogue.lines = lines.clone();
            dialogue.index = 0;
        }
    }
}

fn update_dialogue_ui(
    dialogue: Res<Dialogue>,
    assets: Res<AssetServer>,
    mut box_visible: Single<&mut Visibility, With<DialogueBox>>,
    mut face: Single<(&mut Node, &mut ImageNode), With<DialogueFace>>,
    mut name: Single<&mut Text, (With<DialogueName>, Without<DialogueFace>)>,
    mut text: Single<&mut Text, (With<DialogueText>, Without<DialogueName>)>,
) {
    if !dialogue.is_changed() {
        return;
    }
    if !dialogue.is_active() {
        **box_visible = Visibility::Hidden;
        return;
    }

    let line = &dialogue.lines[dialogue.index];
    **box_visible = Visibility::Inherited;
    text.0 = line.text.clone();
    let (face_node, face_img) = &mut *face;
    if let Some(f) = &line.face {
        face_node.display = Display::Flex;
        face_img.image = assets.load(f.clone());
    } else {
        face_node.display = Display::None;
    }
    name.0 = line.speaker.clone();
}
