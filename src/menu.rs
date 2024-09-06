use bevy::prelude::*;
use bevy::render::camera::ScalingMode::WindowSize;
use bevy_nine_slice_ui::{NineSliceUiMaterialBundle, NineSliceUiPlugin, NineSliceUiTexture};
use bevy_pancam::PanCam;

use crate::assets::UiAssets;
use crate::ENTITY_SIZE_IN_PIXELS;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NineSliceUiPlugin::default())
            .add_systems(OnEnter(crate::states::States::Menu), setup_menu)
            .add_systems(
                Update,
                (button_style_system, play_button_clicked_system).run_if(in_state(crate::states::States::Menu)),
            )
            .add_systems(OnExit(crate::states::States::Menu), cleanup_menu)
            .add_systems(OnEnter(crate::states::States::Worldgen), setup_load_play_ui)
            .add_systems(OnExit(crate::states::States::LoadPlay), cleanup_load_play_ui);
    }
}

#[derive(Component)]
struct Menu;

fn setup_menu(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands
        .spawn(Camera2dBundle {
            // projection: OrthographicProjection {
            //     scaling_mode: WindowSize(ENTITY_SIZE_IN_PIXELS),
            //     ..default()
            // },
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
        background_color: BackgroundColor(Color::rgb_u8(253, 246, 227)), // Solarized Base3
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

    let mut middle = left.clone();
    middle.style.align_items = AlignItems::Center;
    middle.style.justify_content = JustifyContent::Center;
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
                format!("Plowpaw ({}) ({})", env!("VERGEN_GIT_BRANCH"), env!("VERGEN_GIT_SHA")),
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        })
        .id();

    let middle_id = commands.spawn(middle).id();

    let play_button_id = commands
        .spawn(ButtonBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(120.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NineSliceUiMaterialBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        display: Display::Flex,
                        ..default()
                    },
                    nine_slice_texture: NineSliceUiTexture::from_slice(
                        ui_assets.buttons_image.clone(),
                        Rect::new(0., 0., 48., 48.),
                    ),
                    ..default()
                })
                .with_children(|children| {
                    children.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font_size: 48.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
        })
        .insert(bevy::prelude::Name::new("Play Button"))
        .insert(PlayButton)
        .id();

    let exit_button_id = commands
        .spawn(ButtonBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(120.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NineSliceUiMaterialBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        display: Display::Flex,
                        ..default()
                    },
                    nine_slice_texture: NineSliceUiTexture::from_slice(
                        ui_assets.buttons_image.clone(),
                        Rect::new(0., 0., 48., 48.),
                    ),
                    ..default()
                })
                .with_children(|children| {
                    children.spawn(TextBundle::from_section(
                        "Exit",
                        TextStyle {
                            font_size: 48.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
        })
        .insert(bevy::prelude::Name::new("Exit Button"))
        .id();

    commands
        .entity(middle_id)
        .push_children(&[play_button_id, exit_button_id]);

    let right_id = commands.spawn(right).id();
    let _root_id = commands
        .spawn((Menu, bevy::prelude::Name::new("Menu"), root))
        .push_children(&[left_id, middle_id, right_id]);
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct LoadMenu;

fn setup_load_play_ui(mut commands: Commands) {
    let start = std::time::Instant::now();

    commands.spawn((
        NodeBundle {
            style: Style {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb_u8(238, 232, 213)), // Solarized Base2
            ..default()
        },
        LoadMenu,
    ));

    info!("returned in {}ms", start.elapsed().as_millis());
}

fn cleanup_load_play_ui(mut commands: Commands, menu: Query<Entity, With<LoadMenu>>) {
    let start = std::time::Instant::now();

    for entity in menu.iter() {
        commands.entity(entity).despawn_recursive();
    }

    info!("returned in {}ms", start.elapsed().as_millis());
}

fn button_style_system(
    ui_assets: Res<UiAssets>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut nine_slice_query: Query<&mut NineSliceUiTexture>,
) {
    for (interaction, children) in &mut interaction_query {
        let child = children.first().unwrap();
        let mut nine_slice = nine_slice_query.get_mut(*child).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *nine_slice =
                    NineSliceUiTexture::from_slice(ui_assets.buttons_image.clone(), Rect::new(48., 48., 96., 96.));
            }
            Interaction::Hovered => {
                *nine_slice =
                    NineSliceUiTexture::from_slice(ui_assets.buttons_image.clone(), Rect::new(0., 144., 48., 192.));
            }
            Interaction::None => {
                *nine_slice =
                    NineSliceUiTexture::from_slice(ui_assets.buttons_image.clone(), Rect::new(0., 96., 48., 144.));
            }
        }
    }
}

#[derive(Component)]
struct PlayButton;

fn play_button_clicked_system(
    interactions: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    mut next_state: ResMut<NextState<crate::states::States>>,
) {
    for interaction in interactions.iter() {
        match interaction {
            Interaction::Pressed => next_state.set(crate::states::States::Worldgen),
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
