use godot::prelude::*;
use godot::classes::{Area2D, IArea2D, Line2D};

/// The aiming visual — an `Area2D` that draws a line from the stone's
/// resting centre to the current drag position.
///
/// Input handling lives in `GameManager`; this class only manages the
/// `Line2D` child so the GameManager can call `show_line` / `hide_line`.
#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Aim {
    /// Centre of the aim circle (stone's initial position).
    centre: Vector2,

    base: Base<Area2D>,
}

#[godot_api]
impl Aim {
    #[func]
    pub fn set_centre(&mut self, centre: Vector2) {
        self.centre = centre;
    }

    #[func]
    pub fn get_centre(&self) -> Vector2 {
        self.centre
    }

    /// Show a line from centre to `to`.
    #[func]
    pub fn show_line(&mut self, to: Vector2) {
        if let Some(mut line) = self.base().try_get_node_as::<Line2D>("Line2D") {
            line.set_points(&PackedVector2Array::from(vec![self.centre, to]));
            line.set_default_color(Color::BLACK);
            line.set_width(2.0);
            line.show();
        }
    }

    /// Hide the aim line.
    #[func]
    pub fn hide_line(&mut self) {
        if let Some(mut line) = self.base().try_get_node_as::<Line2D>("Line2D") {
            line.hide();
            line.set_points(&PackedVector2Array::new());
        }
    }
}

#[godot_api]
impl IArea2D for Aim {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            centre: Vector2::new(320.0, 480.0),
            base,
        }
    }

    fn ready(&mut self) {
        // Aim area has no collision for physics — it's purely visual.
        self.base_mut().set_monitoring(false);

        // Hide the line initially.
        self.hide_line();
    }
}
