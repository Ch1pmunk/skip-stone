use godot::prelude::*;
use godot::classes::{Area2D, IArea2D, Node2D};

use crate::ball::Ball;

const SKIP_ANGLE_DEG: f32 = 20.0;
const SKIP_SPEED_MS: f32 = 4.0;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Water {
    scroll_offset: f32,
    base: Base<Area2D>,
}

#[godot_api]
impl Water {
    fn apply_water_physics(&self, mut ball: Gd<Ball>) {
        let vel = ball.bind().get_ball_velocity();
        let speed = vel.length();
        let mut angle = vel.angle_to(Vector2::RIGHT).abs().to_degrees();
        if angle > 90.0 { angle = 180.0 - angle; }

        if angle <= SKIP_ANGLE_DEG && speed >= SKIP_SPEED_MS * 100.0 {
            let new_vx = vel.x * 0.8;
            let new_vy = vel.y * -0.8;
            ball.bind_mut().apply_velocity(Vector2::new(new_vx, new_vy));
        } else {
            let mut b = ball.bind_mut();
            b.base_mut().set_gravity_scale(0.2);
            b.apply_velocity(Vector2::new(vel.x * 0.5, vel.y));
        }
    }

    #[func]
    fn on_water_body_entered(&mut self, body: Gd<Node2D>) {
        if let Ok(ball) = body.try_cast::<Ball>() {
            self.apply_water_physics(ball);
        }
    }
}

#[godot_api]
impl IArea2D for Water {
    fn init(base: Base<Area2D>) -> Self {
        Self { scroll_offset: 0.0, base }
    }

    fn ready(&mut self) {
        self.base_mut().set_collision_mask(1u32);
        self.base_mut().set_monitoring(true);
        self.signals()
            .body_entered()
            .connect_self(Water::on_water_body_entered);
    }

    fn process(&mut self, delta: f64) {
        self.scroll_offset += delta as f32 * 60.0;
        let cycle = 16.0;
        if self.scroll_offset >= cycle { self.scroll_offset -= cycle; }
        self.base_mut().queue_redraw();
    }

    fn draw(&mut self) {
        // Only draw the portion visible to the camera, in Water's local coords.
        let water_x = self.base().get_global_position().x;
        let cam_local_x = self
            .base()
            .get_viewport()
            .and_then(|vp| vp.get_camera_2d())
            .map(|cam| cam.get_global_position().x - water_x)
            .unwrap_or(0.0);

        let margin = 800.0;
        let left = cam_local_x - margin;
        let right = cam_local_x + margin;

        let black = Color::from_rgb(0.0, 0.0, 0.0);
        let dash = 8.0;
        let gap = 8.0;
        let pattern = dash + gap;

        // Align to dash pattern.
        let mut x = (left / pattern).floor() * pattern + self.scroll_offset;

        while x < right {
            let end = (x + dash).min(right);
            self.base_mut()
                .draw_line_ex(Vector2::new(x, 0.0), Vector2::new(end, 0.0), black)
                .width(4.0)
                .done();
            x += pattern;
        }
    }
}
