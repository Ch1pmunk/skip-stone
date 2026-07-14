use godot::prelude::*;
use godot::classes::{IRigidBody2D, Node, RigidBody2D, rigid_body_2d::FreezeMode};

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
pub struct Ball {
    pub is_launched: bool,
    pub total_distance: f32,
    last_x: f32,
    pub dist_since_accel: f32,
    base: Base<RigidBody2D>,
}

#[godot_api]
impl Ball {
    #[signal]
    pub fn hit_ground();
    #[signal]
    pub fn distance_changed(distance_m: f32);

    #[func]
    pub fn reset(&mut self, pos: Vector2) {
        self.is_launched = false;
        self.total_distance = 0.0;
        self.last_x = pos.x;
        self.dist_since_accel = 0.0;

        let mut base = self.base_mut();
        base.set_gravity_scale(1.0);
        base.set_freeze_enabled(true);
        base.set_global_position(pos);
        base.set_linear_velocity(Vector2::ZERO);
    }

    #[func]
    pub fn launch(&mut self, velocity: Vector2) {
        self.last_x = self.base().get_global_position().x;
        self.is_launched = true;
        // First accelerator at 1300 px of travel (just off-screen right).
        self.dist_since_accel = 700.0;

        let mut base = self.base_mut();
        base.set_freeze_enabled(false);
        base.set_linear_velocity(velocity);
    }

    #[func] pub fn get_flight_distance(&self) -> f32 { self.total_distance }
    #[func] pub fn reset_accel_counter(&mut self) { self.dist_since_accel = 0.0; }
    #[func] pub fn get_dist_since_accel(&self) -> f32 { self.dist_since_accel }

    #[func]
    pub fn apply_velocity(&mut self, vel: Vector2) {
        self.base_mut().set_linear_velocity(vel);
    }

    pub fn get_ball_velocity(&self) -> Vector2 {
        self.base().get_linear_velocity()
    }

    #[func]
    fn on_body_entered(&mut self, body: Gd<Node>) {
        if body.is_in_group("ground") {
            self.base_mut().set_linear_velocity(Vector2::ZERO);
            self.base_mut().set_gravity_scale(0.0);
            self.signals().hit_ground().emit();
        }
    }
}

#[godot_api]
impl IRigidBody2D for Ball {
    fn init(base: Base<RigidBody2D>) -> Self {
        Self {
            is_launched: false,
            total_distance: 0.0,
            last_x: 0.0,
            dist_since_accel: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_collision_layer(1u32);
        self.base_mut().set_collision_mask(2u32);
        self.base_mut().set_contact_monitor(true);
        self.base_mut().set_max_contacts_reported(1);
        self.base_mut().set_lock_rotation_enabled(true);

        self.base_mut().set_freeze_enabled(true);
        self.base_mut().set_freeze_mode(FreezeMode::KINEMATIC);

        self.signals()
            .body_entered()
            .connect_self(Ball::on_body_entered);

        self.last_x = self.base().get_global_position().x;
        self.base_mut().queue_redraw();
    }

    fn physics_process(&mut self, _delta: f64) {
        if !self.is_launched { return; }

        let pos = self.base().get_global_position();
        let dx = pos.x - self.last_x;
        self.total_distance += dx;
        self.dist_since_accel += dx.abs();
        self.last_x = pos.x;

        let dist_m = self.total_distance / 100.0;
        self.signals().distance_changed().emit(dist_m);
        self.base_mut().queue_redraw();
    }

    fn draw(&mut self) {
        self.base_mut()
            .draw_circle_ex(Vector2::ZERO, 20.0, Color::from_rgb(1.0, 1.0, 1.0))
            .filled(true).done();
        self.base_mut()
            .draw_circle_ex(Vector2::ZERO, 20.0, Color::from_rgb(0.0, 0.0, 0.0))
            .filled(false).width(1.5).done();
    }
}
