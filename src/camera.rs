use crate::constants::{PAN_LIMIT_X, PAN_LIMIT_Y, PAN_START_THRESHOLD_PX, ZOOM_MAX, ZOOM_MIN};
use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource, Default)]
pub struct PinchZoom {
    pub prev_distance: Option<f32>,
}

#[derive(Resource, Default)]
pub struct TouchPan {
    pub prev_position: Option<Vec2>,
    pub dragging: bool,
}

pub fn pinch_zoom_camera(
    touches: Res<Touches>,
    mut pinch: ResMut<PinchZoom>,
    mut cameras: Query<&mut Projection, With<MainCamera>>,
) {
    let positions: Vec<Vec2> = touches.iter().map(|touch| touch.position()).collect();
    if positions.len() != 2 {
        pinch.prev_distance = None;
        return;
    }

    let distance = positions[0].distance(positions[1]);
    if distance <= f32::EPSILON {
        return;
    }

    if let Some(prev) = pinch.prev_distance {
        let Ok(mut projection) = cameras.single_mut() else {
            return;
        };
        if let Projection::Orthographic(ortho) = projection.as_mut() {
            // Fingers apart → distance grows → scale shrinks → zoom in.
            ortho.scale = (ortho.scale * (prev / distance)).clamp(ZOOM_MIN, ZOOM_MAX);
        }
    }

    pinch.prev_distance = Some(distance);
}

pub fn touch_pan_camera(
    touches: Res<Touches>,
    mut pan: ResMut<TouchPan>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut Transform), With<MainCamera>>,
) {
    let touch_count = touches.iter().count();
    if touch_count != 1 {
        pan.prev_position = None;
        pan.dragging = false;
        return;
    }

    let Some(touch) = touches.iter().next() else {
        return;
    };
    let pos = touch.position();

    let Some(prev) = pan.prev_position else {
        pan.prev_position = Some(pos);
        return;
    };
    pan.prev_position = Some(pos);

    let screen_delta = pos - prev;
    if !pan.dragging {
        if screen_delta.length() < PAN_START_THRESHOLD_PX {
            return;
        }
        pan.dragging = true;
    }

    let Ok((camera, global_transform, mut transform)) = cameras.single_mut() else {
        return;
    };

    let Ok(prev_world) = camera.viewport_to_world_2d(global_transform, prev) else {
        return;
    };
    let Ok(curr_world) = camera.viewport_to_world_2d(global_transform, pos) else {
        return;
    };

    // Content follows the finger (camera moves opposite to drag).
    let world_delta = prev_world - curr_world;
    transform.translation.x =
        (transform.translation.x + world_delta.x).clamp(-PAN_LIMIT_X, PAN_LIMIT_X);
    transform.translation.y =
        (transform.translation.y + world_delta.y).clamp(-PAN_LIMIT_Y, PAN_LIMIT_Y);
}
