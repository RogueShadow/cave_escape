use std::collections::HashMap;
use neo_granseal::mesh::{FillStyle, rect_filled};
use neo_granseal::prelude::*;
use crate::cave::CaveEvent;

#[derive(Default)]
pub struct TitleScreen {
    meshes: HashMap<String,Mesh>
}

impl NeoGransealEventHandler for TitleScreen {
    fn event(&mut self, core: &mut NGCore, event: Event) {
        match event {
            Event::KeyEvent {key, state} => {
                if state == KeyState::Pressed && key == Key::Space {
                    core.event(CaveEvent::SetScene(1));
                }
            }
            Event::Draw => {
                let angle = core.timer.elapsed().as_secs_f32();
                let origin = vec2(core.config.width / 2,core.config.height / 2);
                let screen = rect_filled(vec2(0,0),vec2(core.config.width,core.config.height),FillStyle::Solid(Color::BLACK));
                let mut g = ShapeGfx::new(core);
                g.draw_mesh(&screen,vec2(0,0));
                g.rotate(angle);
                g.set_rotation_origin(origin);
                g.draw_mesh(&self.meshes["title"],vec2(0,0));
            }
            Event::Load => {
                let font = Font::default();
                let mut mb = MeshBuilder::default();
                let mut title = font.text("Cave");
                title.scale(19f32);
                title.solid(Color::WHITE);
                mb.solid(Color::new(0.1,0.1,0.1,1.0));
                mb.rounded_rect(vec2(title.width() + 32f32,title.height() + 32f32),8f32);
                let boxed_title = mb.build();

                title.translate(vec2(boxed_title.width() / 2f32 - (title.width() / 2f32),boxed_title.height() / 2f32 + title.height() / 2f32));
                let mut boxed_title = boxed_title.add(&title);
                boxed_title.translate(
                    vec2(
                        core.config.width as f32 / 2f32 - boxed_title.width() / 2f32,
                        core.config.height as f32 / 2f32 - boxed_title.height() / 2f32,
                    )
                );
                boxed_title.buffer();
                let image = core.load_image_from_memory(include_bytes!("../assets/granseal.png")).expect("Load image");
                boxed_title.texture(&image,true);

                self.meshes.insert("title".to_owned(),boxed_title);
            }
            _ => {}
        }
    }
}