use crate::*;

#[derive(Component, Clone)]
pub struct MouseLook {
    mouse_lock: bool,
    pub rotation_sensitivity: f32,
    yaw: f32, 
    pitch: f32,
}

impl MouseLook {
    pub fn new() -> Self {
        Self {
            mouse_lock: false,
            rotation_sensitivity: 0.005,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
    pub fn unlock(&mut self, app: &mut KappApplication) {
        app.set_cursor_visible(true);
        self.mouse_lock = false;
        app.unlock_mouse_position();
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

                mouse_look.yaw -= difference.x * mouse_look.rotation_sensitivity;
                if mouse_look.yaw > std::f32::consts::TAU {
                    mouse_look.yaw -= std::f32::consts::TAU;
                }
                if mouse_look.yaw < 0.0 {
                    mouse_look.yaw += std::f32::consts::TAU;
                }

                mouse_look.pitch -= difference.y * mouse_look.rotation_sensitivity;
                
                let angle_allowed_looking_up = 0.4;
                let angle_allowed_looking_down = 0.28;

                if mouse_look.pitch > std::f32::consts::TAU * angle_allowed_looking_up {
                    mouse_look.pitch = std::f32::consts::TAU * angle_allowed_looking_up
                }
                if mouse_look.pitch < -std::f32::consts::TAU * angle_allowed_looking_down {
                    mouse_look.pitch = -std::f32::consts::TAU * angle_allowed_looking_down
                }

                transform.rotation = Quat::from_yaw_pitch_roll(mouse_look.yaw, mouse_look.pitch, 0.0);
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
