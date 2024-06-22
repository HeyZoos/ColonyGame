use bevy::prelude::*;
use bevy_pancam::PanCam;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(crate::states::States::Menu), setup_menu)
            .add_systems(
                Update,
                click_play_button.run_if(in_state(crate::states::States::Menu)),
            )
            .add_systems(OnExit(crate::states::States::Menu), cleanup_menu);
    }
}

#[derive(Component)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb(0.15, 0.15, 0.15),
            hovered: Color::rgb(0.25, 0.25, 0.25),
        }
    }
}

#[derive(Component)]
struct Menu;

fn setup_menu(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 5.0),
            ..default()
        })
        .insert(PanCam::default());

    // This is the root flex container, from here we'll divide it into thirds
    let root = NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            ..default()
        },
        // background_color: BackgroundColor(Color::rgb(1.0, 1.0, 1.0)),
        ..default()
    };

    let left = NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            flex_shrink: 0.0,
            flex_basis: Val::Px(0.0),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::End,
            padding: UiRect::all(Val::Percent(2.0)),
            ..default()
        },
        // background_color: BackgroundColor(Color::rgb(1.0, 0.0, 0.0)),
        ..default()
    };

    let middle = left.clone();
    // middle.background_color = BackgroundColor(Color::rgb(0.0, 1.0, 0.0));

    let mut right = left.clone();
    // right.background_color = BackgroundColor(Color::rgb(0.0, 0.0, 1.0));
    right.style.align_items = AlignItems::End;
    right.style.justify_content = JustifyContent::End;
    right.style.padding = UiRect::all(Val::Percent(2.0));

    let left_id = commands
        .spawn(left)
        .with_children(|children| {
            children.spawn(TextBundle::from_section(
                format!(
                    "Plowpaw ({}) ({})",
                    env!("VERGEN_GIT_BRANCH"),
                    env!("VERGEN_GIT_SHA")
                ),
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        })
        .id();

    let middle_id = commands.spawn(middle).id();
    let right_id = commands.spawn(right).id();
    let _root_id = commands
        .spawn((Menu, Name::new("Menu"), root))
        .push_children(&[left_id, middle_id, right_id]);
}

#[derive(Component)]
struct ChangeState(crate::states::States);

#[derive(Component)]
struct OpenLink(&'static str);

fn click_play_button(
    mut next_state: ResMut<NextState<crate::states::States>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonColors,
            Option<&ChangeState>,
            Option<&OpenLink>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, button_colors, change_state, open_link) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(state) = change_state {
                    next_state.set(state.0.clone());
                } else if let Some(link) = open_link {
                    if let Err(error) = webbrowser::open(link.0) {
                        warn!("Failed to open link {error:?}");
                    }
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
