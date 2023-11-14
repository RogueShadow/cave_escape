#![warn(clippy::pedantic)]

use std::collections::HashMap;
use std::time::{Duration, Instant};
use neo_granseal::mesh::{fill_path_fan};
use neo_granseal::prelude::*;
use neo_granseal::util::{LineSegment, PathBuilder, raycast, Rectangle};
use crate::cave_scene::Cave;
use crate::title_scene::TitleScreen;


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



pub enum UiEvent {
    Click {
        position: Vec2,
        id: String,
    }
}

pub struct UiStyle {
    background: Color,
    foreground: Color,
    text: Color,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            background: Color::DIM_GRAY,
            foreground: Color::WHITE,
            text: Color::BLACK,
        }
    }
}

fn match_ui_event(ui: &mut Ui, core: &mut NGCore, event: &Event) -> bool {
    let pos = core.state.mouse.pos;
    match event {
        Event::KeyEvent { .. } => {
            false
        }
        Event::MousePressed { .. } => {
            match ui {
                Ui::Frame { id, size, position, children, ..} => {
                    children.iter_mut().for_each(|child| {
                        match_ui_event(child,core,event);
                    });
                    let rect = Rectangle::new2(*position,*size);
                    if rect.contains_point(&pos) {
                        core.event(CaveEvent::Ui(UiEvent::Click {position: pos - *position, id: id.to_string()}));
                    }
                    false
                }
                Ui::Label { .. } => {false}
            }
        }
        Event::MouseMoved(x,y) => {
            let pos = vec2(*x, *y);
            match ui {
                Ui::Frame {id, size, position, children, hover} => {
                    let rect = Rectangle::new2(*position, *size);
                    if rect.contains_point(&pos) {

                    }
                }
                Ui::Label { .. } => {}
            }
            false
        }
        _ => {false}
    }
}
pub struct UiThing {
    pub(crate) ui: Ui
}
impl UiThing {
    pub fn event(&mut self, core: &mut NGCore, event: &Event) -> bool {
        match_ui_event(&mut self.ui,core,event)
    }
    pub fn draw(&self, mb: &mut MeshBuilder) {
        let style = UiStyle::default();
        mb.push();
        draw_ui(&style,&self.ui,mb);
        mb.pop();
    }
}

pub fn draw_ui(style: &UiStyle, ui: &Ui, mb: &mut MeshBuilder) {
    match ui {
        Ui::Frame {id, position, size, children, hover} => {
            mb.set_cursor(*position);
            mb.solid(style.background);
            mb.rect(*size);
            children.iter().for_each(|child|draw_ui(&style,child, mb))
        }
        Ui::Label {id, position, size, text, hover } => {}
    }
}

pub enum Ui {
    Frame {
        id: String,
        position: Vec2,
        size: Vec2,
        children: Vec<Ui>,
        hover: bool,
    },
    Label {
        id: String,
        position: Vec2,
        size: Vec2,
        text: String,
        hover: bool,
    }
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
                        match ui {
                            UiEvent::Click { position, id } => {
                                println!("Clicked id {} at {:?}",id,position);
                            }
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
#[derive(Default)]
pub struct MapInfo {
    pub(crate) tiles: HashMap<(i32, i32),TileType>,
    pub(crate) floor: Mesh,
    pub(crate) walls: Mesh,
    pub(crate) collision: Vec<LineSegment>,
    pub(crate) player_start: Vec2,
    pub(crate) objects: Vec<CaveObject>,
}

pub fn build_map(data: &str) -> MapInfo {
    let mut tiles = HashMap::new();
    let door_g = data.lines().next().unwrap().split(',')
        .map(|v| v.parse::<i32>().unwrap()).collect::<Vec<_>>();
    let mut door = 0;

    let mut player_start = Vec2::ZERO;
    let mut objects: Vec<CaveObject> = vec![];

    let mut wall_builder = MeshBuilder::default();
    wall_builder.solid(Color::GRAY);

    let mut floor_builder = MeshBuilder::default();

    floor_builder.set_cursor(vec2(-1000,-1000));
    floor_builder.solid(Color::BLACK);
    floor_builder.rect(vec2(10000,10000));

    let floor_color = Color::rgb_u8(150,77,0);

    #[derive(Default)]
    struct Cell {
        north: Option<usize>,
        west: Option<usize>,
        east: Option<usize>,
        south: Option<usize>,
    }
    let mut cells: HashMap<(usize,usize),Cell> = HashMap::new();

    data.lines().skip(1).enumerate().for_each(|(y,line)| {
        line.chars().enumerate().for_each(|(x,c)| {
            let size = vec2(TILE_WIDTH,TILE_WIDTH);
            let position = vec2(x as i32 * TILE_WIDTH, y as i32 * TILE_WIDTH);
            wall_builder.set_cursor(position);
            floor_builder.set_cursor(position);
            match c {
                'w' => {
                    tiles.insert((x as i32,y as i32),TileType::Wall);
                    cells.insert((x,y),Cell::default());
                    wall_builder.push();
                    wall_builder.solid(Color::rgb(0.05,0.09,0.06));
                    wall_builder.rect(size);
                    wall_builder.pop();
                }
                '.' => {
                    tiles.insert((x as i32,y as i32),TileType::Floor);
                    floor_builder.solid(floor_color); floor_builder.rect(size);

                }
                'g' => {
                    tiles.insert((x as i32,y as i32),TileType::Gold);
                    floor_builder.rect(size);
                    objects.push(CaveObject::Gold(position));
                }
                'd' => {
                    tiles.insert((x as i32,y as i32),TileType::Door);
                    floor_builder.solid(floor_color);
                    floor_builder.rect(size);

                    objects.push(CaveObject::Door(position,door_g[door]));
                    door += 1;
                }
                'b' => {
                    tiles.insert((x as i32,y as i32),TileType::Floor);
                    floor_builder.solid(Color::new(0.25,0.0,0.0,1.0)); floor_builder.rect(size); 
                }
                's' => {
                    tiles.insert((x as i32,y as i32),TileType::Spikes);
                    floor_builder.solid(Color::ORANGE_RED); floor_builder.rect(size); 
                }
                'h' => {
                    tiles.insert((x as i32,y as i32),TileType::Health);
                    floor_builder.solid(floor_color);
                    floor_builder.rect(size);
                    objects.push(CaveObject::Health(position));
                }
                'p' => {
                    tiles.insert((x as i32,y as i32),TileType::Floor);
                    floor_builder.rect(size); player_start = position;
                }
                _ => { }
            }
        });
    });

    let mut collision: Vec<LineSegment> = vec![];

    data.lines().skip(1).enumerate().for_each(|(y,line)| {
        line.chars().enumerate().filter(|(_,q)| q == &'w').for_each(|(x,_)| {
            let (wcell,ecell,ncell,scell) = (&(x-1,y),&(x+1,y),&(x,y-1),&(x,y+1));
            let pos = vec2(x as i32 * TILE_WIDTH, y as i32 * TILE_WIDTH);
            let mut cell = Cell::default();
            // Should I make a western edge?
            if !cells.contains_key(wcell) {
                // Is there a western edge north of me I can extend?
                if !cells.contains_key(ncell) {
                    // No, let's make one.
                    let edge = LineSegment::new(pos + vec2(0,TILE_WIDTH), pos );
                    collision.push(edge);
                    cell.west = Some(collision.len() - 1);
                } else {
                    // Yes, Does it have an edge to extend?
                    if let Some(edge) = cells.get(ncell).unwrap().west {
                        collision[edge].begin.y += TILE_WIDTH as f32;
                        cell.west = Some(edge);
                    } else {
                        // No, let's make one.
                        let edge = LineSegment::new(pos + vec2(0,TILE_WIDTH), pos );
                        collision.push(edge);
                        cell.west = Some(collision.len() - 1);
                    }
                }
            }
            // Northern Edge
            if !cells.contains_key(ncell) {
                if !cells.contains_key(wcell) {
                    let edge = LineSegment::new(pos , pos+ vec2(TILE_WIDTH, 0));
                    collision.push(edge);
                    cell.north = Some(collision.len() - 1);
                } else if let Some(edge) = cells.get(wcell).unwrap().north {
                    collision[edge].end.x += TILE_WIDTH as f32;
                    cell.north = Some(edge);
                } else {
                    let edge = LineSegment::new(pos , pos+ vec2(TILE_WIDTH, 0));
                    collision.push(edge);
                    cell.north = Some(collision.len() - 1);
                }
            }
            // Eastern Edge
            if !cells.contains_key(ecell) {
                if !cells.contains_key(ncell) {
                    let edge = LineSegment::new(pos + vec2(TILE_WIDTH,0), pos + vec2(TILE_WIDTH, TILE_WIDTH));
                    collision.push(edge);
                    cell.east = Some(collision.len() - 1);
                } else if let Some(edge) = cells.get(ncell).unwrap().east {
                    collision[edge].end.y += TILE_WIDTH as f32;
                    cell.east = Some(edge);
                } else {
                    let edge = LineSegment::new(pos + vec2(TILE_WIDTH,0), pos + vec2(TILE_WIDTH, TILE_WIDTH));
                    collision.push(edge);
                    cell.east = Some(collision.len() - 1);
                }
            }
            // Southern Edge
            if !cells.contains_key(scell) {
                if !cells.contains_key(wcell) {
                    let edge = LineSegment::new(pos + vec2(TILE_WIDTH,TILE_WIDTH), pos + vec2(0, TILE_WIDTH));
                    collision.push(edge);
                    cell.south = Some(collision.len() - 1);
                } else if let Some(edge) = cells.get(wcell).unwrap().south {
                    collision[edge].begin.x += TILE_WIDTH as f32;
                    cell.south = Some(edge);
                } else {
                    let edge = LineSegment::new(pos + vec2(TILE_WIDTH,TILE_WIDTH), pos + vec2(0, TILE_WIDTH));
                    collision.push(edge);
                    cell.south = Some(collision.len() - 1);
                }
            }

            cells.insert((x,y),cell);
        });
    });

    // collision.iter().for_each(|e| {
    //     wall_builder.set_cursor(Vec2::ZERO);
    //     wall_builder.solid(Color::RED);
    //     wall_builder.line(e.begin,e.end);
    //     let nor = e.normal() * 10f32;
    //     wall_builder.fade_left(Color::SPRING_GREEN,Color::YELLOW);
    //     wall_builder.line(e.begin + e.axis() * (e.length()/2f32), e.begin + e.axis() * (e.length()/2f32) + nor);
    //     wall_builder.solid(Color::GREEN);
    //     wall_builder.set_cursor(e.begin - vec2(2,2));
    //     wall_builder.rect(vec2(3,3));
    //     wall_builder.set_cursor(e.end - vec2(2,2));
    //     wall_builder.rect(vec2(3,3));
    // });

    let floor = floor_builder.build();
    floor.buffer();

    let walls = wall_builder.build();
    walls.buffer();

    MapInfo {
        tiles,
        floor,
        walls,
        collision,
        player_start,
        objects,
    }
}

pub fn raycast_for_light(origin: &Vec2, walls: &[LineSegment]) -> Mesh {
    let cull = SCREEN * 0.55;

    let targets = walls.iter().map(|w|
        w.begin
    ).filter(|t| (t.x - origin.x).abs() <= cull.x && (t.y - origin.y).abs() <= cull.y ).collect::<Vec<_>>();

    let mut hits = vec![];
    targets.iter().for_each(|v| {
        let target_angle = (*v - *origin).angle2();
        let (angle1,angle2) = (target_angle - 0.0005,target_angle + 0.0005);
        if let Some(wall_hit) = raycast(*origin,angle_vec2(angle1),walls) {
            hits.push(wall_hit.hit);
        }
        if let Some(wall_hit) = raycast(*origin,angle_vec2(angle2),walls) {
            hits.push(wall_hit.hit);
        }
    });

    hits.sort_by(|a,b| {
        (*origin - *a).angle2().total_cmp(&(*origin - *b).angle2())
    });
    let mut mb = MeshBuilder::default();
    mb.solid(Color::TRANSPARENT);
    mb.set_thickness(TILE_WIDTH as f32 / 1.2);
    let mut pb = PathBuilder::default();
    for (i, hit) in hits.iter().enumerate() {
        if i == 0 {
            pb.move_to(*hit);
            mb.set_cursor(*hit);
            mb.rect(vec2(16,16) - vec2(8,8));
        } else {
            pb.line_to(*hit);
            mb.set_cursor(*hit - vec2(8,8));
            mb.rect(vec2(16,16));

        }
    }
    pb.close_path(true);
    let path = pb.build();

    mb.stroke_path(&path);

    let light = fill_path_fan(&origin,&path);
    let mut light = light.add(&mb.build());
    //let mut light = triangle_fan(origin,hits);

    light.solid(Color::TRANSPARENT);
    light
}
