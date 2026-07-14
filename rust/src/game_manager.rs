use godot::prelude::*;
use godot::classes::{
    Area2D, Button, Camera2D, CanvasLayer, CollisionShape2D, Input, INode2D, Label, Panel,
    Node2D, RectangleShape2D, StyleBoxFlat, VisibleOnScreenEnabler2D,
};
use godot::global::MouseButton;

use crate::aim::Aim;
use crate::ball::Ball;
use rand::Rng;

// ── Constants ───────────────────────────────────────────────────────────
// Units: 1 m = 100 px

const AIM_RADIUS: f32 = 90.0;
const LAUNCH_THRESHOLD: f32 = 15.0;    // px;  y = 0.2x → 3 m/s = 300 px/s at threshold
const DRAG_SCALE: f32 = 20.0;          // v(px/s) = drag_px × 20  (= 0.2 × 100)

const BALL_SPAWN: Vector2 = Vector2::new(320.0, 480.0);

const M_PER_PX: f32 = 0.01;            // 1 px = 0.01 m
const ACCEL_SPAWN_INTERVAL: f32 = 2000.0; // 20 m = 2000 px

// Camera soft-box (screen coords of allowed ball position).
const CAM_LOW_X: f32 = 256.0;
const CAM_HIGH_X: f32 = 1024.0;
const CAM_LOW_Y: f32 = 144.0;
const CAM_HIGH_Y: f32 = 576.0;

const RESULT_DELAY_SEC: f32 = 1.0;

// ── State ───────────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq)]
enum GameState { Aiming, Flying, Grounded, GameOver }

// ── GameManager ─────────────────────────────────────────────────────────

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct GameManager {
    #[init(node = "Ball")]
    ball: OnReady<Gd<Ball>>,
    #[init(node = "Aim")]
    aim: OnReady<Gd<Aim>>,
    #[init(node = "Camera2D")]
    camera: OnReady<Gd<Camera2D>>,

    distance_label: Option<Gd<Label>>,
    result_panel: Option<Gd<Panel>>,

    #[init(val = GameState::Aiming)]
    state: GameState,
    #[init(val = 0.0)]
    best_distance: f32,

    #[init(val = 0.0)]
    ground_hit_time: f64,
    #[init(val = false)]
    is_dragging: bool,

    base: Base<Node2D>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Inherent methods
// ═══════════════════════════════════════════════════════════════════════════

#[godot_api]
impl GameManager {
    fn on_ball_hit_ground(&mut self) {
        self.state = GameState::Grounded;
        self.ground_hit_time = 0.0;
    }
    fn on_ball_distance_changed(&mut self, _dist_m: f32) {
        self.refresh_distance_ui();
    }

    // ── Restart ───────────────────────────────────────────────────

    #[func]
    fn restart(&mut self) {
        let accels = self.base().get_tree().get_nodes_in_group("accelerator");
        for n in accels.iter_shared() {
            self.base_mut().remove_child(&n);
        }
        self.ball.bind_mut().reset(BALL_SPAWN);
        self.aim.bind_mut().hide_line();
        self.camera.set_global_position(Vector2::new(640.0, 360.0));

        if let Some(ref mut p) = self.result_panel { p.hide(); }

        self.state = GameState::Aiming;
        self.ground_hit_time = 0.0;
        self.is_dragging = false;
        self.refresh_distance_ui();
    }

    // ── UI ────────────────────────────────────────────────────────

    fn refresh_distance_ui(&mut self) {
        let m = self.ball.bind().get_flight_distance() * M_PER_PX;
        if let Some(ref mut label) = self.distance_label {
            label.set_text(format!("{:+0.0} m", m).as_str());
        }
    }

    fn show_result_ui(&mut self) {
        let m = self.ball.bind().get_flight_distance() * M_PER_PX;
        if m > self.best_distance { self.best_distance = m; }

        if let Some(ref mut panel) = self.result_panel {
            let kids = panel.get_children();
            if kids.len() > 0 {
                if let Some(mut l) = kids.get(0).and_then(|n| n.try_cast::<Label>().ok()) {
                    l.set_text(format!("距离: {:+0.0} m", m).as_str());
                }
            }
            if kids.len() > 1 {
                if let Some(mut l) = kids.get(1).and_then(|n| n.try_cast::<Label>().ok()) {
                    l.set_text(format!("历史最高: {:.0} m", self.best_distance).as_str());
                }
            }
            panel.show();
        }
        self.state = GameState::GameOver;
    }

    // ── Accelerator ───────────────────────────────────────────────

    fn check_spawn_accelerator(&mut self) {
        if self.ball.bind().get_dist_since_accel() >= ACCEL_SPAWN_INTERVAL {
            self.ball.bind_mut().reset_accel_counter();
            self.spawn_accelerator();
        }
    }

    fn spawn_accelerator(&mut self) {
        let mut rng = rand::thread_rng();
        let rng_y: f32 = rng.gen_range(0.0..=600.0);

        let mut area = Area2D::new_alloc();
        let id: i64 = rng.gen_range(0..100000);
        area.set_name(format!("Accel_{}", id).as_str());
        let ball_x = self.ball.bind().base().get_global_position().x;
        area.set_global_position(Vector2::new(ball_x + 1300.0, rng_y));
        area.set_collision_layer(3u32);
        area.set_collision_mask(1u32);
        area.set_monitoring(true);
        area.add_to_group("accelerator");

        let mut shape = CollisionShape2D::new_alloc();
        let mut rect = RectangleShape2D::new_gd();
        rect.set_size(Vector2::new(150.0, 150.0));
        shape.set_shape(&rect);
        area.add_child(&shape);
        shape.set_owner(&area);

        let mut label = Label::new_alloc();
        label.set_text(">>");
        label.set_position(Vector2::new(-55.0, -45.0));
        label.add_theme_font_size_override("font_size", 80);
        label.set("theme_override_colors/font_color", &Color::from_rgb(0.0, 0.0, 0.0).to_variant());
        area.add_child(&label);
        label.set_owner(&area);

        // VisibleOnScreenEnabler2D — per spec.
        let enabler = VisibleOnScreenEnabler2D::new_alloc();
        area.add_child(&enabler);

        self.base_mut().add_child(&area);
        area.set_owner(&self.base().get_tree().get_root().unwrap());

        let gd = self.to_gd();
        area.signals()
            .body_entered()
            .builder()
            .connect_other_gd(&gd, |mut gm: Gd<GameManager>, body: Gd<Node2D>| {
                gm.bind_mut().on_accelerator_body_entered(body);
            });
    }

    #[func]
    fn on_accelerator_body_entered(&mut self, body: Gd<Node2D>) {
        if let Ok(mut ball) = body.try_cast::<Ball>() {
            let vel = ball.bind().get_ball_velocity();
            let new_vx = vel.x * 1.1;
            let new_vy = if vel.y < 0.0 { vel.y * 1.1 } else { vel.y * -0.5 };
            ball.bind_mut().apply_velocity(Vector2::new(new_vx, new_vy));
        }
    }

    // ── Aim & input ───────────────────────────────────────────────

    fn handle_aim_input(&mut self) {
        let input = Input::singleton();
        let mouse = self.base().get_global_mouse_position();
        let centre = self.aim.bind().get_centre();
        let btn = input.is_mouse_button_pressed(MouseButton::LEFT);

        // Start drag only when clicking on the ball itself.
        if btn && !self.is_dragging {
            let bp = self.ball.bind().base().get_global_position();
            if (mouse - bp).length() <= 25.0 {
                self.is_dragging = true;
            }
        }
        if self.is_dragging {
            if btn {
                let dir = mouse - centre;
                let len = dir.length();
                let clamped = if len > AIM_RADIUS {
                    centre + dir.normalized() * AIM_RADIUS
                } else {
                    mouse
                };
                self.ball.bind_mut().base_mut().set_global_position(clamped);
                self.aim.bind_mut().show_line(clamped);
            } else {
                // Released.
                self.is_dragging = false;
                let dir = mouse - centre;
                let len = dir.length();
                if len < LAUNCH_THRESHOLD {
                    self.ball.bind_mut().reset(BALL_SPAWN);
                } else {
                    let launch_dir = (centre - mouse).normalized();
                    self.ball.bind_mut().launch(launch_dir * len * DRAG_SCALE);
                    self.state = GameState::Flying;
                }
                self.aim.bind_mut().hide_line();
            }
        }
    }

    // ── Build UI (once, from ready) ───────────────────────────────

    fn build_ui(&mut self) {
        let black = Color::from_rgb(0.0, 0.0, 0.0);
        let bg = Color::from_rgb(0.867, 0.867, 0.867); // #DDDDDD

        let mut ui_layer = CanvasLayer::new_alloc();
        ui_layer.set_name("UILayer");
        ui_layer.set_layer(1);

        let mut dist_label = Label::new_alloc();
        dist_label.set_name("DistanceLabel");
        dist_label.set_text("+0 m");
        dist_label.set_position(Vector2::new(1100.0, 10.0));
        dist_label.add_theme_font_size_override("font_size", 28);
        dist_label.set("theme_override_colors/font_color", &black.to_variant());
        ui_layer.add_child(&dist_label);
        self.distance_label = Some(dist_label);

        // ── Result panel ──────────────────────────────────────────
        let panel_w = 500.0;
        let panel_h = 200.0;
        let panel_x = 640.0 - panel_w / 2.0; // screen centre
        let panel_y = 360.0 - panel_h / 2.0;

        let mut panel = Panel::new_alloc();
        panel.set_name("ResultPanel");
        panel.set_position(Vector2::new(panel_x, panel_y));
        panel.set_size(Vector2::new(panel_w, panel_h));
        panel.hide();

        // Attach drag script.
        let drag_script = godot::tools::load::<godot::classes::GDScript>("res://drag_panel.gd");
        panel.set_script(&drag_script);

        let mut style = StyleBoxFlat::new_gd();
        style.set_bg_color(bg);
        style.set_border_color(black);
        style.set_border_width_all(5);
        style.set_corner_radius_all(12);
        panel.set("theme_override_styles/panel", &style.to_variant());

        // Labels — full width, centred text.
        let mut l1 = Label::new_alloc();
        l1.set_name("CurrentDist");
        l1.set_position(Vector2::new(0.0, 20.0));
        l1.set_size(Vector2::new(panel_w, 30.0));
        l1.set_text("距离: +0 m");
        l1.set_horizontal_alignment(godot::global::HorizontalAlignment::CENTER);
        l1.add_theme_font_size_override("font_size", 28);
        l1.set("theme_override_colors/font_color", &black.to_variant());
        panel.add_child(&l1); l1.set_owner(&panel);

        let mut l2 = Label::new_alloc();
        l2.set_name("BestDist");
        l2.set_position(Vector2::new(0.0, 55.0));
        l2.set_size(Vector2::new(panel_w, 30.0));
        l2.set_text("历史最高: 0 m");
        l2.set_horizontal_alignment(godot::global::HorizontalAlignment::CENTER);
        l2.add_theme_font_size_override("font_size", 28);
        l2.set("theme_override_colors/font_color", &black.to_variant());
        panel.add_child(&l2); l2.set_owner(&panel);

        // Button — centred.
        let btn_w = 200.0;
        let btn_h = 50.0;
        let mut btn = Button::new_alloc();
        btn.set_name("RestartBtn");
        btn.set_position(Vector2::new((panel_w - btn_w) / 2.0, 120.0));
        btn.set_size(Vector2::new(btn_w, btn_h));
        btn.set_text("重新开始");
        btn.add_theme_font_size_override("font_size", 28);
        btn.set("theme_override_colors/font_color", &black.to_variant());

        let mut bs = StyleBoxFlat::new_gd();
        bs.set_bg_color(bg); bs.set_border_color(black);
        bs.set_border_width_all(2); bs.set_corner_radius_all(6);
        btn.set("theme_override_styles/normal", &bs.to_variant());
        btn.set("theme_override_styles/hover", &bs.to_variant());
        btn.set("theme_override_styles/pressed", &bs.to_variant());

        let gd = self.to_gd();
        btn.signals().pressed().builder()
            .connect_other_mut(&gd, |this: &mut Self| { this.restart(); });

        panel.add_child(&btn); btn.set_owner(&panel);
        ui_layer.add_child(&panel);
        self.result_panel = Some(panel);

        self.base_mut().add_child(&ui_layer);
    }

    // ── Camera (hard tracking) ────────────────────────────────────

    /// "硬性跟踪" — the ball must stay within the soft-box [CAM_LOW_X..CAM_HIGH_X]
    /// × [CAM_LOW_Y..CAM_HIGH_Y] in screen space. The camera moves only when
    /// the ball would leave this zone.
    fn update_camera(&mut self) {
        if self.state == GameState::Aiming { return; }

        let bp = self.ball.bind().base().get_global_position();
        let mut cx = self.camera.get_global_position().x;
        let mut cy = self.camera.get_global_position().y;

        let sx = bp.x - cx + 640.0; // ball screen-x (camera centre = 640)
        let sy = bp.y - cy + 360.0; // ball screen-y (camera centre = 360)

        // Push camera only when ball leaves the soft-box.
        if sx < CAM_LOW_X { cx -= CAM_LOW_X - sx; }
        if sx > CAM_HIGH_X { cx += sx - CAM_HIGH_X; }
        if sy < CAM_LOW_Y { cy -= CAM_LOW_Y - sy; }
        if sy > CAM_HIGH_Y { cy += sy - CAM_HIGH_Y; }

        self.camera.set_global_position(Vector2::new(cx, cy));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Node2D virtual methods
// ═══════════════════════════════════════════════════════════════════════════

#[godot_api]
impl INode2D for GameManager {
    fn ready(&mut self) {
        self.build_ui();
        let gd = self.to_gd();
        self.ball.signals().hit_ground().connect_other(&gd, Self::on_ball_hit_ground);
        self.ball.signals().distance_changed().connect_other(&gd, Self::on_ball_distance_changed);
        self.ball.bind_mut().reset(BALL_SPAWN);
        self.aim.bind_mut().set_centre(BALL_SPAWN);

        if let Some(mut g) = self.base().try_get_node_as::<Node2D>("Ground") {
            g.add_to_group("ground");
        }
        godot_print!("[Skip-Stone] Ready!");
    }

    fn process(&mut self, delta: f64) {
        self.update_camera();
        match self.state {
            GameState::Aiming   => self.handle_aim_input(),
            GameState::Flying   => self.check_spawn_accelerator(),
            GameState::Grounded => {
                self.ground_hit_time += delta;
                if self.ground_hit_time >= RESULT_DELAY_SEC as f64 {
                    self.show_result_ui();
                }
            }
            GameState::GameOver => {
                // Restart only via the button's pressed signal.
            }
        }
    }
}
