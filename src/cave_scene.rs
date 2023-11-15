use std::collections::HashMap;
use std::time::Instant;
use neo_granseal::mesh::fill_path_fan;
use neo_granseal::prelude::*;
use neo_granseal::util::{LineSegment, PathBuilder, raycast};
use crate::cave::{ CaveEvent, CaveObject, Player, SCREEN, TileType};
use crate::TILE_WIDTH;

pub struct Cave {
    cam: Camera,
    player: Player,
    map: HashMap<(i32,i32),TileType>,
    meshes: HashMap<&'static str,Mesh>,
    collision: Vec<LineSegment>,
    objects: Vec<CaveObject>,
    font: Font,
    images: HashMap<String,Image>,
    colors: HashMap<&'static str,Ani<Color>>,
}
impl Default for Cave {
    fn default() -> Self {
        Self {
            cam: Camera::new(Vec2::ZERO),
            player: Player::new(),
            map: HashMap::new(),
            meshes: HashMap::new(),
            collision: vec![],
            objects: vec![],
            font: Font::new(64f32),
            images: HashMap::new(),
            colors: HashMap::new(),
        }
    }
}
impl NeoGransealEventHandler for Cave {
    fn event(&mut self, core: &mut NGCore, event: Event) {
        match event {
            Event::KeyEvent {state,key} => {
                if state == KeyState::Pressed && key == Key::F1 {
                    core.event(CaveEvent::SetScene(0));
                }
            }
            Event::Draw => {
                let time = core.timer.elapsed().as_secs_f32();
                let mut g = ShapeGfx::new(core);
                let mut mb = MeshBuilder::default();
                g.set_offset(-self.cam.get_offset()); // now g draws in world space instead of screen space.

                g.draw_mesh(&self.meshes["darkness"],self.cam.get_offset());
                g.draw_mesh(&self.meshes["light"], self.cam.get_offset());
                g.render_image(&self.images["base"],true);

                g.draw_mesh(&self.meshes["floor"], Vec2::ZERO);
                self.map.iter().for_each(|(p,t)|{
                    let pos = vec2(p.0 * TILE_WIDTH,p.1 * TILE_WIDTH);
                    match t {
                        TileType::Gold => {
                            mb.solid(self.colors["gold"].ani(time));
                            mb.set_cursor(pos);
                            mb.rect(vec2(TILE_WIDTH,TILE_WIDTH));
                        }
                        TileType::Door => {
                            mb.solid(self.colors["door"].ani(time));
                            mb.set_cursor(pos);
                            mb.rect(vec2(TILE_WIDTH,TILE_WIDTH));
                        }
                        TileType::Health => {
                            mb.solid(self.colors["health"].ani(time));
                            mb.set_cursor(pos);
                            mb.rect(vec2(TILE_WIDTH,TILE_WIDTH));
                        }
                        TileType::Spikes => {
                            mb.solid(self.colors["spikes"].ani(time));
                            mb.set_cursor(pos);
                            mb.rect(vec2(TILE_WIDTH,TILE_WIDTH));
                        }
                        _ => {}
                    }
                });
                g.draw_mesh(&mb.build(),Vec2::ZERO);
                g.set_tint(self.colors["player"].ani(time));
                g.draw_mesh(&self.meshes["player"],self.player.ani.ani(time));
                g.set_tint(self.colors["walls"].ani(time));
                g.draw_mesh(&self.meshes["walls"], Vec2::ZERO);
                g.draw_image(&self.images["base"],Vec2::ZERO);
                //g.draw_mesh(&self.meshes["light"], Vec2::ZERO);
                //g.draw_mesh(&self.meshes["debug"],Vec2::ZERO);

                g.set_offset(Vec2::ZERO); // back to screen space
                let status = self.font.text(
                    format!(
                        "Health: {}\nGold: {}\n{},{}",
                        self.player.health,
                        self.player.gold,
                        self.player.pos.x.floor() as i32 / TILE_WIDTH,
                        self.player.pos.y.floor() as i32 / TILE_WIDTH,
                    ).as_str(),1f32
                );
                g.set_tint(Color::ORANGE);
                g.draw_mesh(&status,vec2(16,16f32 + status.max_y()));
                g.set_tint(Color::WHITE);


                g.draw_mesh(&mb.build(), Vec2::ZERO);

            }
            Event::Update(_) => {
                let time = core.timer.elapsed().as_secs_f32();
                let player = &mut self.player;
                if player.frozen_timer.elapsed() >= player.freeze_time {
                    let mut new_pos = player.pos;
                    if core.key_held(Key::A) {
                        player.frozen_timer = Instant::now();
                        new_pos.x -= TILE_WIDTH as f32;
                    }
                    if core.key_held(Key::S) {
                        player.frozen_timer = Instant::now();
                        new_pos.y += TILE_WIDTH as f32;
                    }
                    if core.key_held(Key::D) {
                        player.frozen_timer = Instant::now();
                        new_pos.x += TILE_WIDTH as f32;
                    }
                    if core.key_held(Key::W) {
                        player.frozen_timer = Instant::now();
                        new_pos.y -= TILE_WIDTH as f32;
                    }
                    let t_pos = (new_pos.x.floor() as i32 / TILE_WIDTH,new_pos.y.floor() as i32 / TILE_WIDTH);
                    let default = &mut TileType::Wall;
                    let t_type = self.map.get_mut(&t_pos).unwrap_or(default);
                    match t_type {
                        TileType::Floor => {
                            player.ani = Ani::new(time,player.freeze_time.as_secs_f32(),vec![player.pos,new_pos]);
                            player.ani.repeat = false;
                            player.pos = new_pos;
                        }
                        TileType::Wall => {}
                        TileType::Door => {
                            if player.gold >= 5 {
                                player.gold -= 5;
                                self.map.insert(t_pos,TileType::Floor);
                                player.ani = Ani::new(time,player.freeze_time.as_secs_f32(),vec![player.pos,new_pos]);
                                player.ani.repeat = false;
                                player.pos = new_pos;
                            }
                        }
                        TileType::Gold => {
                            self.map.insert(t_pos,TileType::Floor);
                            player.gold += 1;
                            player.ani = Ani::new(time,player.freeze_time.as_secs_f32(),vec![player.pos,new_pos]);
                            player.ani.repeat = false;
                            player.pos = new_pos;
                        }
                        TileType::Health => {
                            self.map.insert(t_pos,TileType::Floor);
                            player.health += 1;
                            player.ani = Ani::new(time,player.freeze_time.as_secs_f32(),vec![player.pos,new_pos]);
                            player.ani.repeat = false;
                            player.pos = new_pos;
                        }
                        TileType::Warp => {}
                        TileType::Exit => {}
                        TileType::Spikes => {
                            player.health -= 1;
                        }
                    }
                }

                //core.set_title(format!("Cave: {} :: {}",core.state.fps,delta));

                let obj_collision = self.map.iter().flat_map(|(pos,t)|{
                    let pos = match t {
                        TileType::Door |
                        TileType::Spikes => {Some(vec2(pos.0 * TILE_WIDTH, pos.1 * TILE_WIDTH))}
                        _ => {None}
                    };
                    if let Some(p) = pos {
                        let top = LineSegment::new(p,p + vec2(TILE_WIDTH,0));
                        let right = LineSegment::new(p + vec2(TILE_WIDTH,0), p + vec2(TILE_WIDTH,TILE_WIDTH));
                        let bottom = LineSegment::new(p + vec2(TILE_WIDTH,TILE_WIDTH), p + vec2(0,TILE_WIDTH));
                        let left = LineSegment::new(p + vec2(0,TILE_WIDTH), p);
                        Some([top,right,bottom,left])
                    } else {None}
                }).flatten().collect::<Vec<_>>();

                let mut collision: Vec<LineSegment> = vec![];
                let screen = vec2(core.config.height,core.config.width);
                collision.extend(&self.collision);
                let start = self.player.pos - screen / 2.0;
                collision.extend(&vec!(
                    LineSegment::new(start,start + vec2(screen.x,0)).reverse_normal(),
                    LineSegment::new(start + vec2(screen.x,0),start + screen).reverse_normal(),
                    LineSegment::new(start + screen,start + vec2(0,screen.y)).reverse_normal(),
                    LineSegment::new(start + vec2(0,screen.y),start).reverse_normal(),
                ));
                collision.extend(obj_collision);
                self.meshes.insert("light", raycast_for_light(&(self.player.ani.ani(time) + vec2(TILE_WIDTH,TILE_WIDTH) / 2f32),&collision));
                self.cam.target(self.player.ani.ani(time) - vec2(core.config.width / 2, core.config.height / 2));
            }
            Event::Load => {
                self.colors.insert("floor",
                                   Ani::new(0.0,3.0,vec![Color::rgb_u8(150,77,0)])
                );
                self.colors.insert("walls",
                                   Ani::new(0.0,3.0,vec![Color::DARK_GRAY,Color::rgb_u8(100, 25,0),Color::DARK_GRAY])
                );
                self.colors.insert("gold",
                                   Ani::new(0.0,3.0,vec![Color::ORANGE,Color::YELLOW,Color::ORANGE])
                );
                self.colors.insert("warp",
                                   Ani::new(0.0,3.0,vec![Color::BLUE,Color::CYAN,Color::GREEN,Color::BLUE])
                );
                self.colors.insert("health",
                                   Ani::new(0.0,3.0,vec![Color::GREEN,Color::WHITE,Color::rgb_u8(0,150,50),Color::GREEN])
                );
                self.colors.insert("blood",
                                   Ani::new(0.0,3.0,vec![Color::new(0.25,0.0,0.0,1.0)])
                );
                self.colors.insert("exit",
                                   Ani::new(0.0,3.0,vec![Color::MAGENTA,Color::WHITE,Color::MAGENTA])
                );
                self.colors.insert("door",
                                   Ani::new(0.0,1.0,vec![Color::rgb_u8(130,20,0),Color::rgb_u8(170,70,20),Color::rgb_u8(130,20,0)])
                );
                self.colors.insert("player",
                                   Ani::new(0.0,3.0,vec![Color::BLUE,Color::CYAN,Color::BLUE])
                );
                self.colors.insert("spikes",
                                   Ani::new(0.0,1.0,vec![Color::RED,Color::BLACK,Color::ORANGE,Color::RED])
                );
                let mut mb = MeshBuilder::default();
                mb.solid(Color::BLACK);
                mb.rect(vec2(8192,8192));
                let darkness = mb.build();
                self.meshes.insert("darkness",darkness);
                let base = core.create_image(8192,8192);
                self.images.insert("base".to_owned(),base);
                let map = build_map(include_str!("../assets/map.txt"));
                self.map = map.tiles;
                self.meshes.insert("floor",map.floor);
                self.meshes.insert("walls",map.walls);
                mb.rounded_rect(vec2(TILE_WIDTH,TILE_WIDTH),4f32);
                self.meshes.insert("player",mb.build());

                self.player.pos = map.player_start;
                self.player.health = 5;
                self.collision = map.collision;
                self.collision.iter().for_each(|l|{
                    l.visualize(&mut mb);
                });
                let debug = mb.build();
                debug.buffer();
                self.meshes.insert("debug", debug);
                self.objects = map.objects;
                self.meshes.insert("light", raycast_for_light(&(self.player.pos + vec2(TILE_WIDTH,TILE_WIDTH) / 2f32),&self.collision));
            }
            _ => {}
        }
    }
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
