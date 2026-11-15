use crate::sequence::{
    ChordModeButton, SelectedNotesEntry, SequenceModeButton, spawn_selected_notes_panel,
};
use crate::tuning::{tuning, tunings};
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseWheel;
use bevy::input::touch::Touches;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;

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

/// Keeps the root font size in sync with the window width. Every `rem` value
/// in the UI (text and spacing) follows this, so the chrome stays proportional
/// from a phone to a desktop.
pub fn sync_rem_size(window: Single<&Window>, mut rem: ResMut<RemSize>) {
    let viewport_min = window.width().min(window.height());
    let target = crate::constants::rem_size_for_viewport(viewport_min);
    if (rem.0 - target).abs() > 0.01 {
        **rem = target;
    }
}

pub fn spawn_tuning_dropdown(parent: &mut ChildSpawnerCommands, current_index: usize) {
    let current_name: &'static str = tuning(current_index).name;

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                flex_shrink: 0.0,
                ..default()
            },
            GlobalZIndex(20),
            Pickable::IGNORE,
        ))
        .with_children(|dropdown| {
            dropdown
                .spawn((
                    TuningMenuButton,
                    Node {
                        padding: UiRect::axes(Val::Rem(0.6), Val::Rem(0.3)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        min_height: Val::Px(34.0),
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
                            font_size: FontSize::Rem(1.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TuningMenuLabel,
                    ));
                });

            // Absolute panel: the menu floats over the page without pushing it.
            dropdown
                .spawn((
                    TuningMenuPanel,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Percent(100.0),
                        right: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        min_width: Val::Rem(7.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    GlobalZIndex(30),
                    Visibility::Hidden,
                ))
                .with_children(|panel| {
                    for (index, preset) in tunings().iter().enumerate() {
                        panel
                            .spawn((
                                TuningOption { index },
                                Node {
                                    padding: UiRect::axes(Val::Rem(0.6), Val::Rem(0.4)),
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Center,
                                    min_height: Val::Px(36.0),
                                    width: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.22, 0.22, 0.28)),
                            ))
                            .with_children(|opt| {
                                opt.spawn((
                                    Text::new(preset.name),
                                    TextFont {
                                        font_size: FontSize::Rem(1.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}

pub fn spawn_power_chord_popup(commands: &mut Commands) {
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
            GlobalZIndex(100),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Rem(0.6),
                        padding: UiRect::all(Val::Rem(1.0)),
                        max_width: Val::Vw(92.0),
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
                            font_size: FontSize::Rem(1.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        SelectionPopupTitle,
                    ));

                    panel.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Rem(0.8),
                            ..default()
                        },
                        TextColor(Color::srgb(0.85, 0.85, 0.9)),
                        PowerChordPopupText,
                    ));

                    spawn_action_button(panel, "OK", DismissPowerChordPopup);
                });
        });
}

pub fn spawn_chord_controls(parent: &mut ChildSpawnerCommands) {
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
                    padding: UiRect::all(Val::Rem(0.35)),
                    border: UiRect::all(Val::Px(1.0)),
                    row_gap: Val::Rem(0.3),
                    max_width: Val::Vw(94.0),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.92)),
                BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
            ))
            .with_children(|parent| {
                spawn_selected_notes_panel(parent);

                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Rem(0.3),
                        flex_wrap: FlexWrap::Wrap,
                        flex_shrink: 0.0,
                        ..default()
                    },))
                    .with_children(|row| {
                        spawn_action_button(row, "Play", PlayButton);
                        spawn_action_button(row, "Explain", ExplainButton);
                        spawn_action_button(row, "Clear", ClearButton);
                    });

                parent.spawn((
                    Text::new("Select up to 6 notes, then Play or Explain."),
                    TextLayout::linebreak(LineBreak::WordBoundary),
                    TextFont {
                        font_size: FontSize::Rem(0.9),
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

fn spawn_action_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Bundle) {
    parent
        .spawn((
            marker,
            Node {
                padding: UiRect::axes(Val::Rem(0.6), Val::Rem(0.3)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                min_height: Val::Px(34.0),
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
                    font_size: FontSize::Rem(1.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn toggle_tuning_menu(
    presses: Query<(), (Changed<Pressed>, With<TuningMenuButton>)>,
    mut panels: Query<&mut Visibility, With<TuningMenuPanel>>,
) {
    for () in &presses {
        for mut visibility in &mut panels {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

pub fn dismiss_power_chord_popup(
    presses: Query<(), (Changed<Pressed>, With<DismissPowerChordPopup>)>,
    mut popups: Query<&mut Visibility, With<PowerChordPopup>>,
) {
    for () in &presses {
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
    mut pages: Query<(&Hovered, &mut ScrollPosition), With<PageScroll>>,
) {
    let total: f32 = wheel_messages.read().map(|event| event.y).sum();
    if total == 0.0 {
        return;
    }
    for (hovered, mut scroll) in &mut pages {
        if hovered.get() {
            scroll.y -= total * SCROLL_LINES_PER_TICK;
        }
    }
}

/// Finger currently dragging the page on a touch screen.
#[derive(Resource, Default)]
pub struct PageTouchScroll {
    finger: Option<u64>,
    scrolling: bool,
}

/// Scrolls the page by dragging a finger over it. Gestures that start on a
/// note, button or the scrollbar belong to that widget, so they are ignored.
#[allow(clippy::type_complexity)]
pub fn touch_scroll_page(
    touches: Res<Touches>,
    // Only real controls (notes, buttons, chips, scrollbar) block the gesture.
    // Plain layout nodes are pressed by any touch and must not count.
    pressed: Query<
        (),
        (
            With<Pressed>,
            Or<(
                With<crate::fretboard::FretNote>,
                With<PlayButton>,
                With<ExplainButton>,
                With<ClearButton>,
                With<TuningMenuButton>,
                With<TuningOption>,
                With<ChordModeButton>,
                With<SequenceModeButton>,
                With<SelectedNotesEntry>,
                With<DismissPowerChordPopup>,
                With<PageScrollTrack>,
                With<PageScrollThumb>,
            )>,
        ),
    >,
    mut state: ResMut<PageTouchScroll>,
    mut pages: Query<&mut ScrollPosition, With<PageScroll>>,
) {
    if state.finger.is_none()
        && let Some(touch) = touches.iter_just_pressed().next()
    {
        state.finger = Some(touch.id());
        state.scrolling = pressed.is_empty();
    }

    let Some(id) = state.finger else {
        return;
    };
    let released = touches.just_released(id) || touches.just_canceled(id);

    if state.scrolling
        && let Some(touch) = touches.get_pressed(id)
    {
        let dy = touch.delta().y;
        if dy != 0.0 {
            for mut scroll in &mut pages {
                scroll.y -= dy;
            }
        }
    }

    if released {
        state.finger = None;
        state.scrolling = false;
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
        (&ComputedNode, &UiGlobalTransform, Has<Pressed>),
        (With<PageScrollTrack>, Without<PageScrollThumb>),
    >,
    thumbs: Query<Has<Pressed>, With<PageScrollThumb>>,
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
    for (track_node, track_transform, track_pressed) in &tracks {
        let Some(normalized) = track_node.normalize_point(*track_transform, cursor) else {
            continue;
        };
        let fraction = (0.5 - normalized.y).clamp(0.0, 1.0);
        let thumb_pressed = thumbs.iter().any(|pressed| pressed);
        if thumb_pressed || track_pressed {
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
