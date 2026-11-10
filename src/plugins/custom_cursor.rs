use bevy::camera::ClearColorConfig;
use bevy::camera::visibility::RenderLayers;
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::window::{CursorMoved, CursorOptions, PrimaryWindow};

mod lesson_magnifier;
mod theme_balloon;

/// Provides custom cursors for exploration and lesson screens.
pub struct CustomCursorPlugin;

impl Plugin for CustomCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            follow_pointer_system.run_if(resource_exists::<CustomCursorActive>),
        );
        theme_balloon::register(app);
        lesson_magnifier::register(app);
    }
}

#[derive(Component, Reflect)]
#[require(Transform, Visibility)]
struct PointerCursor;

#[derive(Component, Default, Reflect)]
struct TouchCursorState {
    touches: Vec<(u64, Vec2)>,
    active_touch: Option<u64>,
    last_touch: Option<u64>,
    last_position: Option<Vec2>,
}

impl TouchCursorState {
    fn update_touch(&mut self, phase: TouchPhase, id: u64, position: Vec2) {
        match phase {
            TouchPhase::Started => {
                if let Some((_, touch_position)) = self
                    .touches
                    .iter_mut()
                    .find(|(touch_id, _)| *touch_id == id)
                {
                    *touch_position = position;
                } else {
                    self.touches.push((id, position));
                }

                if self.active_touch.is_none() {
                    self.active_touch = Some(id);
                }
            }
            TouchPhase::Moved => {
                if let Some((_, touch_position)) = self
                    .touches
                    .iter_mut()
                    .find(|(touch_id, _)| *touch_id == id)
                {
                    *touch_position = position;
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                let was_active = self.active_touch == Some(id);
                if was_active {
                    self.last_position = Some(position);
                }
                self.touches.retain(|(touch_id, _)| *touch_id != id);

                if was_active {
                    self.active_touch = self.touches.first().map(|(touch_id, _)| *touch_id);
                    self.last_touch = Some(id);
                }
            }
        }

        if let Some(active_touch) = self.active_touch {
            self.last_position = self
                .touches
                .iter()
                .find(|(touch_id, _)| *touch_id == active_touch)
                .map(|(_, position)| *position)
                .or(self.last_position);
            self.last_touch = Some(active_touch);
        }
    }

    fn displayed_pointer_id(&self) -> PointerId {
        self.active_touch
            .or(self.last_touch)
            .map_or(PointerId::Mouse, PointerId::Touch)
    }

    fn cursor_position(
        &mut self,
        mouse_moved: bool,
        window_position: Option<Vec2>,
    ) -> Option<Vec2> {
        if self.active_touch.is_some() {
            self.last_position
        } else if mouse_moved {
            self.last_position = None;
            self.last_touch = None;
            window_position
        } else {
            self.last_position.or(window_position)
        }
    }
}

/// Marks the overlay camera that renders cursor visuals above the UI.
#[derive(Component, Reflect)]
struct CursorCamera;

#[derive(Resource, Reflect)]
struct CustomCursorActive;

const CURSOR_RENDER_LAYER: usize = 1;

fn activate_custom_cursor(commands: &mut Commands, cursor_options: &mut CursorOptions) {
    commands.insert_resource(CustomCursorActive);
    cursor_options.visible = false;
}

fn cleanup_custom_cursor(
    mut commands: Commands,
    active: Option<Res<CustomCursorActive>>,
    cursor_query: Query<Entity, With<PointerCursor>>,
    camera_query: Query<Entity, With<CursorCamera>>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if active.is_none() {
        return;
    }

    cursor_options.visible = true;

    for entity in &cursor_query {
        commands.entity(entity).despawn();
    }
    for entity in &camera_query {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<CustomCursorActive>();
}

fn spawn_cursor_camera(commands: &mut Commands, cursor_layer: &RenderLayers) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        cursor_layer.clone(),
        CursorCamera,
    ));
}

fn follow_pointer_system(
    primary_window: Single<(Entity, &Window), With<PrimaryWindow>>,
    camera_data: Single<(&Camera, &GlobalTransform), With<CursorCamera>>,
    cursor_query: Single<
        (&mut Transform, &mut TouchCursorState, &mut Visibility),
        With<PointerCursor>,
    >,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut touch_events: MessageReader<TouchInput>,
) {
    let (window_entity, window) = *primary_window;
    let (mut transform, mut touch_state, mut visibility) = cursor_query.into_inner();

    let mut mouse_moved = false;
    for event in cursor_moved_events.read() {
        if event.window == window_entity {
            mouse_moved = true;
        }
    }

    for touch in touch_events.read() {
        if touch.window == window_entity {
            touch_state.update_touch(touch.phase, touch.id, touch.position);
        }
    }

    let cursor_position = touch_state.cursor_position(mouse_moved, window.cursor_position());
    if !window.focused {
        *visibility = Visibility::Hidden;
        return;
    }

    let Some(cursor_position) = cursor_position else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Visible;

    let (camera, camera_transform) = *camera_data;
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    transform.translation.x = world_pos.x;
    transform.translation.y = world_pos.y;
}
