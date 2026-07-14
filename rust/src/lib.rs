use godot::prelude::*;

mod aim;
mod ball;
mod game_manager;
mod water;

struct SkipStone;

#[gdextension]
unsafe impl ExtensionLibrary for SkipStone {}
