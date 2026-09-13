use crate::constants::FONT_SIZE;
use crate::sequence::spawn_selected_notes_panel;
use crate::tuning::{tuning, tunings};
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseWheel;
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

pub fn spawn_chord_controls(
    parent: &mut ChildSpawnerCommands,
    font_size: f32,
    info_font_size: f32,
) {
    let panel_padding = if font_size < FONT_SIZE { 6.0 } else { 12.0 };
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0,
                align_self: AlignSelf::Center,
                align_items: AlignItems::Center,
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
                    padding: UiRect::all(Val::Px(panel_padding)),
                    border: UiRect::all(Val::Px(1.0)),
                    row_gap: Val::Px(8.0),
                    max_width: Val::Vw(90.0),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.92)),
                BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
            ))
            .with_children(|parent| {
                spawn_selected_notes_panel(parent, font_size, info_font_size);

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
                    TextLayout::linebreak(LineBreak::WordBoundary),
                    TextFont {
                        font_size: FontSize::Px(info_font_size),
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.85, 0.9)),
                    ChordInfoText,
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_shrink: 0.0,
                        ..default()
                    },
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

/// Marker for the vertical scroll container wrapping the whole page.
#[derive(Component)]
pub struct PageScroll;

/// Marker for the scrollbar track (right edge of the page).
#[derive(Component)]
pub struct PageScrollTrack;

/// Marker for the draggable scrollbar thumb.
#[derive(Component)]
pub struct PageScrollThumb;

const SCROLL_LINES_PER_TICK: f32 = 48.0;
const SCROLLBAR_THUMB_MIN_PX: f32 = 24.0;

/// Scrolls the page with the mouse wheel while the cursor is over it.
pub fn scroll_page(
    mut wheel_messages: MessageReader<MouseWheel>,
    mut pages: Query<(&Interaction, &mut ScrollPosition), With<PageScroll>>,
) {
    let total: f32 = wheel_messages.read().map(|event| event.y).sum();
    if total == 0.0 {
        return;
    }
    for (interaction, mut scroll) in &mut pages {
        if *interaction == Interaction::Hovered {
            scroll.y -= total * SCROLL_LINES_PER_TICK;
        }
    }
}

/// Sizes and offsets the scrollbar thumb to match the page's scroll position.
/// Hides the thumb entirely when the content fits the page.
#[allow(clippy::type_complexity)]
pub fn update_page_scrollbar(
    pages: Query<(&ComputedNode, &ScrollPosition), With<PageScroll>>,
    mut tracks: Query<(&ComputedNode, &mut Visibility), With<PageScrollTrack>>,
    mut thumbs: Query<
        (&mut Node, &mut Visibility),
        (With<PageScrollThumb>, Without<PageScrollTrack>),
    >,
) {
    let Ok((page, scroll)) = pages.single() else {
        return;
    };
    let max_scroll = (page.content_size.y - page.size.y).max(0.0);
    let fits = max_scroll <= 0.0;

    for (track, mut track_visibility) in &mut tracks {
        let target = if fits {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if *track_visibility != target {
            *track_visibility = target;
        }
        if fits {
            continue;
        }
        let gutter = track.size().y;
        for (mut node, mut thumb_visibility) in &mut thumbs {
            if *thumb_visibility != Visibility::Visible {
                *thumb_visibility = Visibility::Visible;
            }
            let thumb_h = (gutter * page.size.y / page.content_size.y).max(SCROLLBAR_THUMB_MIN_PX);
            let top = Val::Px(scroll.y / max_scroll * (gutter - thumb_h));
            let height = Val::Px(thumb_h);
            if node.top != top {
                node.top = top;
            }
            if node.height != height {
                node.height = height;
            }
        }
    }
}

/// Drags the scrollbar thumb and jumps on track clicks.
#[allow(clippy::type_complexity)]
pub fn drag_page_scrollbar(
    window: Single<&Window>,
    mut pages: Query<(&ComputedNode, &mut ScrollPosition), With<PageScroll>>,
    tracks: Query<
        (&ComputedNode, &UiGlobalTransform, &Interaction),
        (With<PageScrollTrack>, Without<PageScrollThumb>),
    >,
    thumbs: Query<Option<&Interaction>, With<PageScrollThumb>>,
) {
    let Some(cursor) = window.physical_cursor_position() else {
        return;
    };
    let Ok((page, mut scroll)) = pages.single_mut() else {
        return;
    };
    let max_scroll = (page.content_size.y - page.size.y).max(0.0);
    if max_scroll <= 0.0 {
        return;
    }

    let mut thumb_fraction: Option<f32> = None;
    for (track_node, track_transform, track_interaction) in &tracks {
        let Some(normalized) = track_node.normalize_point(*track_transform, cursor) else {
            continue;
        };
        let fraction = (0.5 - normalized.y).clamp(0.0, 1.0);
        let thumb_pressed = thumbs
            .iter()
            .any(|interaction| matches!(interaction, Some(Interaction::Pressed)));
        if thumb_pressed || *track_interaction == Interaction::Pressed {
            thumb_fraction = Some(fraction);
        }
    }

    if let Some(fraction) = thumb_fraction {
        let new_y = fraction * max_scroll;
        if scroll.y != new_y {
            scroll.y = new_y;
        }
    }
}
