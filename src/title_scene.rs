use neo_granseal::mesh::{FillStyle, rect_filled};
use neo_granseal::prelude::*;
use crate::cave::CaveEvent;
use crate::ui::{Ui, UiImageStyle, UiLabelStyle, UiThing};

#[derive(Default)]
pub struct TitleScreen {
    ui: UiThing,
}

impl NeoGransealEventHandler for TitleScreen {
    fn event(&mut self, core: &mut NGCore, event: Event) {
        if self.ui.event(core,&event) {return}
        match event {
            Event::KeyEvent {key, state} => {
                if state == KeyState::Pressed && key == Key::Space {
                    core.event(CaveEvent::SetScene(1));
                }
            }
            Event::MouseWheel(x,y) => {
                println!("Mouse scroll! {} {}",x,y)
            }
            Event::Draw => {
                let screen = rect_filled(vec2(0,0),vec2(core.config.width,core.config.height),FillStyle::Solid(Color::DARK_CYAN));
                let mut g = ShapeGfx::new(core);
                g.draw_mesh(&screen,vec2(0,0));

                let mut mb = MeshBuilder::default();
                self.ui.draw(&mut mb,&mut g);
                g.draw_mesh(&mb.build(),Vec2::ZERO);
            }
            Event::Load => {
                let gran = core.load_image("assets/granseal.png", true).unwrap();
                let font = Font::default();
                let exit = font.text("Exit Game",100f32).size() + vec2(8,8);
                self.ui.build(&Ui::Frame {
                    name: "title".to_string(),
                    position: vec2(32,32),
                    size: vec2(core.config.width - 64,core.config.height - 64),
                    children: vec![
                        Ui::Label {
                            name: "title".to_string(),
                            position: vec2(16,75),
                            text: "Cave Escape".to_string(),
                            children: vec![],
                            style: UiLabelStyle {
                                text_scale: 150f32,
                                ..Default::default()
                            },
                        },
                        Ui::Label {
                            name: "start".to_string(),
                            position: vec2(100,300),
                            text: "Start Game".to_string(),
                            children: vec![
                                Ui::Image {
                                    name: "granseal".to_string(),
                                    position: vec2(-32,exit.y / 2.0 - 8.0),
                                    image: gran,
                                    style: UiImageStyle::default(),
                                    children: vec![],
                                    size: Some(vec2(16,16)),
                                },
                            ],
                            style: UiLabelStyle {
                                hover_text: FillStyle::FadeLeft(Color::ORANGE_RED,Color::RED),
                                text_scale: 100f32,
                                ..Default::default()
                            },
                        },
                        Ui::Label {
                            name: "exit".to_string(),
                            position: vec2(100,420),
                            text: "Exit Game".to_string(),
                            children: vec![
                                Ui::Image {
                                    name: "granseal2".to_string(),
                                    position: vec2(-32, exit.y / 2.0 - 8.0),
                                    image: gran,
                                    style: UiImageStyle::default(),
                                    children: vec![],
                                    size: Some(vec2(16,16)),
                                },
                            ],
                            style: UiLabelStyle {
                                hover_text: FillStyle::FadeLeft(Color::ORANGE_RED,Color::RED),
                                text_scale: 100f32,
                                ..Default::default()
                            },
                        }
                    ],
                    style: Default::default(),
                });
            }
            _ => {}
        }
    }
}