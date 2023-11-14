mod cave;
mod cave_scene;
mod title_scene;
mod ui;

use neo_granseal::prelude::*;
pub use crate::cave::{SceneHandler, TILE_WIDTH};
use crate::cave_scene::Cave;
use crate::title_scene::TitleScreen;

fn main() {
    let scene_handler = SceneHandler {
        current: 0,
        title: TitleScreen::default(),
        scene1: Cave::default(),
    };
    start(scene_handler, GransealGameConfig::default()
        .size(30 * TILE_WIDTH, 30 * TILE_WIDTH)
        .clear_color(Color::BLACK)
        .vsync(false)
    );
}
