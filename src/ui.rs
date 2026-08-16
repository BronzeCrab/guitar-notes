use crate::constants::{
    CHORD_INFO_MAX_HEIGHT_PX, CHORD_PANEL_INSET_PX, CHORD_PANEL_MAX_WIDTH_PX, FONT_SIZE,
};
use crate::tuning::{tuning, tunings};
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct ExplainButton;

#[derive(Component)]
pub struct ClearButton;

#[derive(Component)]
pub struct ChordInfoText;

#[derive(Component)]
pub struct PowerChordPopup;

#[derive(Component)]
pub struct SelectionPopupTitle;

#[derive(Component)]
pub struct PowerChordPopupText;

#[derive(Component)]
pub struct DismissPowerChordPopup;

#[derive(Component)]
pub struct TuningMenuButton;

#[derive(Component)]
pub struct TuningMenuPanel;

#[derive(Component)]
pub struct TuningMenuLabel;

#[derive(Component)]
pub struct TuningOption {
    pub index: usize,
}

pub fn spawn_tuning_dropdown(commands: &mut Commands, current_index: usize, font_size: f32) {
    let current_name: &'static str = tuning(current_index).name;
    let menu_min_width = if font_size < FONT_SIZE { 120.0 } else { 140.0 };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ZIndex(10),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    TuningMenuButton,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        min_width: Val::Px(menu_min_width),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                    BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("{current_name} v")),
                        TextFont {
                            font_size: FontSize::Px(font_size),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TuningMenuLabel,
                    ));
                });

            parent
                .spawn((
                    TuningMenuPanel,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        min_width: Val::Px(menu_min_width),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    Visibility::Hidden,
                ))
                .with_children(|panel| {
                    for (index, preset) in tunings().iter().enumerate() {
                        panel
                            .spawn((
                                Button,
                                TuningOption { index },
                                Node {
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Center,
                                    width: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.22, 0.22, 0.28)),
                            ))
                            .with_children(|opt| {
                                opt.spawn((
                                    Text::new(preset.name),
                                    TextFont {
                                        font_size: FontSize::Px(font_size),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}

pub fn spawn_power_chord_popup(commands: &mut Commands, font_size: f32) {
    let body_font = if font_size < FONT_SIZE { 15.0 } else { 18.0 };
    commands
        .spawn((
            PowerChordPopup,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ZIndex(100),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        max_width: Val::Px(420.0),
                        width: Val::Percent(92.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.16, 0.2)),
                    BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(font_size),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        SelectionPopupTitle,
                    ));

                    panel.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(body_font),
                            ..default()
                        },
                        TextColor(Color::srgb(0.85, 0.85, 0.9)),
                        PowerChordPopupText,
                    ));

                    spawn_action_button(panel, "OK", DismissPowerChordPopup, font_size);
                });
        });
}

pub fn spawn_chord_controls(commands: &mut Commands, font_size: f32, info_font_size: f32) {
    let panel_padding = if font_size < FONT_SIZE { 8.0 } else { 12.0 };
    let panel_max_height = if font_size < FONT_SIZE { 140.0 } else { 160.0 };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CHORD_PANEL_INSET_PX),
                right: Val::Px(CHORD_PANEL_INSET_PX),
                bottom: Val::Px(CHORD_PANEL_INSET_PX),
                justify_content: JustifyContent::Center,
                padding: UiRect::ZERO,
                ..default()
            },
            ZIndex(10),
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    max_width: Val::Px(CHORD_PANEL_MAX_WIDTH_PX),
                    width: Val::Percent(100.0),
                    max_height: Val::Px(panel_max_height),
                    padding: UiRect::all(Val::Px(panel_padding)),
                    border: UiRect::all(Val::Px(1.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.92)),
                BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        flex_wrap: FlexWrap::Wrap,
                        flex_shrink: 0.0,
                        ..default()
                    },))
                    .with_children(|row| {
                        spawn_action_button(row, "Play", PlayButton, font_size);
                        spawn_action_button(row, "Explain", ExplainButton, font_size);
                        spawn_action_button(row, "Clear", ClearButton, font_size);
                    });

                parent.spawn((
                    Text::new("Select up to 6 notes, then Play or Explain."),
                    TextFont {
                        font_size: FontSize::Px(info_font_size),
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.85, 0.9)),
                    Node {
                        max_width: Val::Percent(100.0),
                        max_height: Val::Px(CHORD_INFO_MAX_HEIGHT_PX),
                        overflow: Overflow::scroll_y(),
                        flex_shrink: 1.0,
                        ..default()
                    },
                    ChordInfoText,
                ));
            });
        });
}

fn spawn_action_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: impl Bundle,
    font_size: f32,
) {
    let min_width = if font_size < FONT_SIZE { 60.0 } else { 72.0 };
    parent
        .spawn((
            Button,
            marker,
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                min_width: Val::Px(min_width),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
            BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(font_size),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn toggle_tuning_menu(
    interactions: Query<&Interaction, (Changed<Interaction>, With<TuningMenuButton>)>,
    mut panels: Query<&mut Visibility, With<TuningMenuPanel>>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for mut visibility in &mut panels {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

pub fn dismiss_power_chord_popup(
    interactions: Query<&Interaction, (Changed<Interaction>, With<DismissPowerChordPopup>)>,
    mut popups: Query<&mut Visibility, With<PowerChordPopup>>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for mut visibility in &mut popups {
            *visibility = Visibility::Hidden;
        }
    }
}
