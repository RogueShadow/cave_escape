#![warn(clippy::pedantic)]

use std::process::exit;
use std::time::{Duration, Instant};
use neo_granseal::prelude::*;
use crate::cave_scene::Cave;
use crate::title_scene::TitleScreen;
use crate::ui::UiEvent;


pub const TILE_WIDTH: i32 = 28;
pub const SCREEN: Vec2 = Vec2 {
    x: TILE_WIDTH as f32 * 30.0,
    y: TILE_WIDTH as f32 * 30.0,
};

pub enum CaveEvent {
    SetScene(usize),
    Error,
    Ui(UiEvent),
}

#[allow(unused)]
pub enum TileType {
    Floor,
    Wall,
    Door,
    Gold,
    Health,
    Warp,
    Exit,
    Spikes,
}


pub struct SceneHandler {
    pub(crate) current: usize,
    pub(crate) title: TitleScreen,
    pub(crate) scene1: Cave,
}





impl NeoGransealEventHandler for SceneHandler {
    fn event(&mut self, core: &mut NGCore, event: Event) {
        match event {
            Event::Load => {
                self.title.event(core, Event::Load);
                self.scene1.event(core,Event::Load);
            }
            Event::Custom(msg) => {
                let msg = *msg.downcast::<CaveEvent>().unwrap_or(Box::new(CaveEvent::Error));
                match msg {
                    CaveEvent::SetScene(scene) => {self.current = scene;}
                    CaveEvent::Error => {println!("Received a strange event, could not unwrap it."); }
                    CaveEvent::Ui(ui) => {
                        println!("{ui:?}");
                        match ui {
                            UiEvent::HoverEnter { .. } => {}
                            UiEvent::HoverExit { .. } => {}
                            UiEvent::MousePressed { id,.. }if id == "start".to_owned() => {
                                core.event(CaveEvent::SetScene(1));
                            }
                            UiEvent::MousePressed { id,.. }if id == "exit".to_owned() => {
                                exit(0);
                            }
                            UiEvent::MouseReleased { .. } => {}
                            _ => {}
                        }
                    }
                }
            }
            Event::Update(_) =>  {
                match self.current {
                    0 => {
                        core.set_title(format!("Title: {}",core.state.fps));
                        self.title.event(core,event);
                    }
                    1 => {
                        core.set_title(format!("Cave: {}",core.state.fps));
                        self.scene1.event(core,event);
                    }
                    _ => {}
                }
            }
            _ => {
                match self.current {
                    0 => {self.title.event(core,event);}
                    1 => {self.scene1.event(core,event);}
                    _ => {}
                }
            }
        }

    }
}



#[derive(Debug)]
pub struct Player {
    pub pos: Vec2,
    pub ani: Ani<Vec2>,
    pub gold: i32,
    pub health: i32,
    pub frozen_timer: std::time::Instant,
    pub freeze_time: std::time::Duration,
}
impl Player {
    pub fn new() -> Self {
        Self {
            pos: Default::default(),
            ani: Ani::new(0.0,1.0,vec![]),
            gold: 0,
            health: 0,
            frozen_timer: Instant::now(),
            freeze_time: Duration::from_secs_f32(0.15),
        }
    }
}

#[derive(Debug)]
pub enum CaveObject {
    Gold(Vec2),
    Door(Vec2,i32),
    Health(Vec2),
}
