use crate::*;

#[derive(Component, Clone)]
pub struct MouseLook {
    mouse_lock: bool,
    pub rotation_sensitivity: f32,
}

impl MouseLook {
    pub fn new() -> Self {
        Self {
            mouse_lock: false,
            rotation_sensitivity: 0.005,
        }
    }
    pub fn fixed_update(
        mut mouse_look: Query<(&mut Transform, &mut Self)>,
        app: &mut KappApplication,
        input: &Input,
    ) {
        for (transform, mouse_look) in mouse_look.iter_mut() {
            if mouse_look.mouse_lock {
                //   let (view_width, view_height) = camera.get_view_size();
                //   let (view_width, view_height) = (view_width as f32, view_height as f32);

                let mouse_motion = input.mouse_motion();
                let difference = Vec2::new(mouse_motion.0 as f32, mouse_motion.1 as f32);

                let rotation_around_y = Quaternion::from_angle_axis(
                    -(difference.x) * mouse_look.rotation_sensitivity,
                    Vec3::Y,
                );
                let rotation_around_x = Quaternion::from_angle_axis(
                    -(difference.y) * mouse_look.rotation_sensitivity,
                    Vec3::X,
                );

                transform.rotation = rotation_around_y * transform.rotation * rotation_around_x;
            }

            if input.pointer_button_down(PointerButton::Primary) {
                if !mouse_look.mouse_lock {
                    app.set_cursor_visible(false);
                    app.lock_mouse_position();
                    mouse_look.mouse_lock = true;
                }
            }

            if input.key(Key::Escape) {
                app.set_cursor_visible(true);
                app.unlock_mouse_position();
                mouse_look.mouse_lock = false;
            }
        }
    }
}
