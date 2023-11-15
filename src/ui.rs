use std::any::Any;
use std::collections::HashMap;
use neo_granseal::prelude::*;
use neo_granseal::util::Rectangle;

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

pub enum UiType {
    Frame,
    Label,
}
#[derive(Default)]
pub struct UiThing {
    pub id_node: HashMap<i32,(Box<dyn Any>, UiType)>,
    pub name_id: HashMap<String,i32>,
    pub parent_children: HashMap<i32,Vec<i32>>,
    pub root: Option<i32>,
}
impl UiThing {
    pub fn event(&mut self, core: &mut NGCore, event: &Event) {
        if let Some(root) = self.root {
            self.match_ui_event(root, core, event, vec2(0,0));
        }
    }
    fn match_ui_event(&mut self, id: i32, core: &mut NGCore, event: &Event, offset: Vec2) -> bool {
        let pos = core.state.mouse.pos;
        let (node,ui_type) = self.id_node.get_mut(&id).unwrap();
        let mut offset = offset;
        match ui_type {
            UiType::Frame => {
                let frame = node.downcast_mut::<UiFrame>().unwrap();
                if Rectangle::new2(frame.position + offset,frame.size).contains_point(&pos) {
                    frame.hover = true;
                } else {
                    frame.hover = false;
                }
                offset += frame.position;
            }
            UiType::Label => {
                let label = node.downcast_mut::<UiLabel>().unwrap();
                if Rectangle::new2(label.position + offset,label.size).contains_point(&pos) {
                    label.hover = true;
                } else {
                    label.hover = false;
                }
                offset += label.position;
            }
        }
        let children = self.parent_children.get(&id).unwrap().iter().map(|c| c.to_owned()).collect::<Vec<_>>();

        children.iter().for_each(|child|{
            self.match_ui_event(*child,core,event,offset);
        });

        match event {
            Event::KeyEvent { .. } => {
                false
            }
            Event::MousePressed { .. } => {
                false
            }
            Event::MouseMoved(x,y) => {
                let pos = vec2(*x, *y);

                false
            }
            _ => {false}
        }
    }
    pub fn draw(&self, mb: &mut MeshBuilder) {
        if let Some(root) = self.root {
            mb.push();
            self.draw_ui(root,mb);
            mb.pop();
        }
    }
    pub fn draw_ui(&self, id: i32, mb: &mut MeshBuilder) {
        let (node,ui_type) = self.id_node.get(&id).unwrap();
        match ui_type {
            UiType::Frame => {
                let frame = node.downcast_ref::<UiFrame>().expect("UiFrame");
                mb.move_cursor(frame.position);
                if frame.hover  {mb.solid(Color::GREEN)} else {mb.solid(Color::DIM_GRAY)};
                mb.set_filled(true);
                mb.rect(frame.size);
                mb.set_filled(false);
                mb.solid(Color::BLACK);
                mb.rect(frame.size);
                let children = self.parent_children.get(&id).unwrap();
                for c_id in children {
                    self.draw_ui(*c_id,mb);
                }
            }
            UiType::Label => {
                let label = node.downcast_ref::<UiLabel>().expect("UiLabel");
                mb.move_cursor(label.position);
                if label.hover  {mb.solid(Color::PINK)} else {mb.solid(Color::CYAN)};
                mb.set_filled(true);
                mb.rect(label.size);
                mb.set_filled(false);
                mb.solid(Color::BLACK);
                mb.rect(label.size);
                let children = self.parent_children.get(&id).unwrap();
                for c_id in children {
                    self.draw_ui(*c_id,mb);
                }
            }
        };
    }
    pub fn get_id(&self) -> i32 {
        let mut id = 0;
        while self.id_node.contains_key(&id) {
            id += 1
        }
        id
    }
    pub fn build(&mut self, ui: &Ui) -> Vec<i32> {
        match ui {
            Ui::Frame {name, position, children, size } => {
                let ids = children.iter().flat_map(|child|{
                    self.build(child)
                }).collect::<Vec<_>>();
                let frame = UiFrame {
                    name: name.to_string(),
                    position: *position,
                    size: *size,
                    ..Default::default()
                };
                let id = self.get_id();
                self.id_node.insert(id, (Box::new(frame),UiType::Frame));
                self.name_id.insert(name.to_string(), id);
                self.parent_children.insert(id,ids);
                if self.root.is_none() {
                    self.root = Some(id);
                }
                vec![id]
            }
            Ui::Label { name,position,size,text,children } => {
                let ids = children.iter().flat_map(|child|{
                    self.build(child)
                }).collect::<Vec<_>>();
                let frame = UiLabel {
                    name: name.to_string(),
                    position: *position,
                    size: *size,
                    text: text.to_string(),
                    ..Default::default()
                };
                let id = self.get_id();
                self.id_node.insert(id, (Box::new(frame),UiType::Label));
                self.name_id.insert(name.to_string(), id);
                self.parent_children.insert(id,ids);
                vec![id]
            }
        }
    }
}
#[derive(Default, Debug)]
pub struct UiFrame {
    name: String,
    position:  Vec2,
    size: Vec2,
    hover: bool,
}
#[derive(Default, Debug)]
pub struct UiLabel {
    name: String,
    position:  Vec2,
    size: Vec2,
    text: String,
    hover: bool,
}

pub enum Ui {
    Frame {
        name: String,
        position: Vec2,
        size: Vec2,
        children: Vec<Ui>,
    },
    Label {
        name: String,
        position: Vec2,
        size: Vec2,
        text: String,
        children: Vec<Ui>,
    }
}
