use std::collections::HashMap;
use neo_granseal::events::MouseButton;
use neo_granseal::mesh::FillStyle;
use neo_granseal::prelude::*;
use neo_granseal::util::Rectangle;
use crate::cave::CaveEvent;

#[derive(Debug)]
pub enum UiEvent {
    MousePressed {
        id: String,
        button: MouseButton,
        position: Vec2,
    },
    MouseReleased {
        id: String,
        button: MouseButton,
        position: Vec2,
    },
    HoverEnter {
        id: String,
        position: Vec2,
    },
    HoverExit {
        id: String,
        position: Vec2,
    },
    MouseScroll {
        id: String,
        scroll: Vec2
    },
}

#[derive(Copy, Clone)]
pub struct UiFrameStyle {
    color: FillStyle,
    hover_color: FillStyle,
    outline: FillStyle,
    hover_outline: FillStyle,
    show_outline: bool,
}
impl Default for UiFrameStyle {
    fn default() -> Self {
        Self {
            color: FillStyle::FadeDown(Color::DIM_GRAY,Color::BLACK),
            hover_color:FillStyle::FadeDown(Color::DIM_GRAY,Color::BLACK),
            outline: FillStyle::Solid(Color::BLACK),
            hover_outline: FillStyle::Solid(Color::BLACK),
            show_outline: true,
        }
    }
}
#[derive(Copy, Clone)]
pub struct UiLabelStyle {
    pub color: FillStyle,
    pub hover_color: FillStyle,
    pub outline: FillStyle,
    pub hover_outline: FillStyle,
    pub text: FillStyle,
    pub hover_text: FillStyle,
    pub show_outline: bool,
    pub show_background: bool,
    pub text_margin: f32,
    pub text_scale: f32,
}
impl Default for UiLabelStyle {
    fn default() -> Self {
        Self {
            color: FillStyle::Solid(Color::DARK_GRAY),
            hover_color: FillStyle::Solid(Color::DARK_GRAY),
            outline: FillStyle::Solid(Color::BLACK),
            hover_outline: FillStyle::Solid(Color::BLACK),
            text: FillStyle::Solid(Color::BLACK),
            hover_text: FillStyle::Solid(Color::BLACK),
            show_outline: false,
            show_background: false,
            text_margin: 4f32,
            text_scale: 24f32,
        }
    }
}
#[derive(Copy,Clone)]
pub struct UiImageStyle {
    pub tint: Color,
    pub scale: Vec2,
}
impl Default for UiImageStyle {
    fn default() -> Self {
        Self {
            tint: Color::WHITE,
            scale: vec2(1,1),
        }
    }
}

pub trait UiComponent {
    fn get_hover(&self) -> bool;
    fn set_hover(&mut self, value: bool);
    fn name(&self) -> String;
    fn get_position(&self) -> Vec2;
    fn contains(&self, point: Vec2) -> bool;
    fn set_dragging(&mut self, value: bool);
    fn is_dragging(&self) -> bool;
    fn set_position(&mut self, value: Vec2);
    fn set_drag_offset(&mut self, value: Vec2);
    fn set_draggable(&mut self, value: bool);
    fn get_drag_offset(&mut self) -> Vec2;
    fn is_draggable(&self) -> bool;
    fn handle_event(&mut self, core: &mut NGCore, event: &Event, offset: Vec2) -> bool {
            match event {
                Event::KeyEvent { .. } => {false}
                Event::MouseWheel(x,y) => {
                    let m_pos = core.state.mouse.pos;
                    if self.contains(m_pos - offset) {
                        core.event(
                            CaveEvent::Ui(
                                UiEvent::MouseScroll {
                                    id: self.name(),
                                    scroll: vec2(*x,*y),
                                }
                            )
                        );
                        true
                    }else{
                        false
                    }
                }
                Event::MousePressed {button,state} => {
                    let m_pos = core.state.mouse.pos;
                    if self.contains(m_pos - offset) {
                        match state {
                            KeyState::Pressed => {
                                if self.is_draggable() {
                                    self.set_dragging(true);
                                    self.set_drag_offset(m_pos - offset - self.get_position());
                                }
                                core.event(
                                    CaveEvent::Ui(
                                        UiEvent::MousePressed {
                                            id: self.name(),
                                            button: button.to_owned(),
                                            position: m_pos,
                                        }
                                    )
                                )
                            }
                            KeyState::Released => {
                                self.set_dragging(false);
                                core.event(
                                    CaveEvent::Ui(
                                        UiEvent::MouseReleased {
                                            id: self.name(),
                                            button: button.to_owned(),
                                            position: m_pos,
                                        }
                                    )
                                )
                            }
                        }
                        true
                    } else {
                        false
                    }
                }
                Event::MouseMoved(x,y) => {
                    let pos = vec2(*x,*y);
                    let doffset = self.get_drag_offset();
                    if self.is_dragging() {
                        self.set_position(pos - offset - doffset);
                    }
                    if self.contains(pos - offset) {
                        if !self.get_hover() {
                            core.event(CaveEvent::Ui(UiEvent::HoverEnter { id: self.name(), position: self.get_position() }));
                        }
                        self.set_hover(true);
                    } else {
                        if self.get_hover() {
                            core.event(CaveEvent::Ui(UiEvent::HoverExit { id: self.name().to_string(), position: self.get_position() }))
                        }
                        self.set_hover(false);
                    }
                    false
                }
                _ => {false}
            }
        }
    fn draw(&self,font: &Font, mb: &mut MeshBuilder, g: &mut ShapeGfx);
}
#[derive(Default)]
pub struct UiThing {
    pub id_node: HashMap<i32,Box<dyn UiComponent>>,
    pub name_id: HashMap<String,i32>,
    pub parent_children: HashMap<i32,Vec<i32>>,
    pub root: Option<i32>,
    pub font: Font,
}
impl UiThing {
    pub fn event(&mut self, core: &mut NGCore, event: &Event) -> bool {
        if let Some(root) = self.root {
            self.match_ui_event(root, core, event, vec2(0,0))
        } else {false}
    }
    fn match_ui_event(&mut self, id: i32, core: &mut NGCore, event: &Event, offset: Vec2) -> bool {
        let node= self.id_node.get_mut(&id).unwrap();
        let children = self.parent_children.get(&id).unwrap().iter().map(|c| c.to_owned()).collect::<Vec<_>>();
        let new_offset = offset + node.get_position().to_owned();
        let mut handled = false;
        children.iter().for_each(|child|{
            if !handled {
                handled = self.match_ui_event(*child,core,event,new_offset);
            }
        });
        if !handled {
            handled = self.id_node.get_mut(&id).unwrap().handle_event(core,event, offset);
        }
        handled
    }
    pub fn draw(&self, mb: &mut MeshBuilder, g: &mut ShapeGfx) {
        if let Some(root) = self.root {
            mb.push();
            self.draw_ui(root,mb,g);
            mb.pop();
        }
    }
    pub fn draw_ui(&self, id: i32, mb: &mut MeshBuilder, g: &mut ShapeGfx) {
        let node= self.id_node.get(&id).unwrap();
        mb.move_cursor(node.get_position());
        node.draw(&self.font, mb,g);
        let children = self.parent_children.get(&id).unwrap();
        for c_id in children {
            self.draw_ui(*c_id,mb,g);
        }
        mb.move_cursor(-node.get_position());
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
            Ui::Frame {name, position, children, size, style } => {
                let ids = children.iter().flat_map(|child|{
                    self.build(child)
                }).collect::<Vec<_>>();
                let frame = UiFrame {
                    name: name.to_string(),
                    position: *position,
                    size: *size,
                    hover: false,
                    style: *style,
                    dragging: false,
                    drag_offset: vec2(0,0),
                    draggable: false,
                };
                let id = self.get_id();
                self.id_node.insert(id, Box::new(frame));
                self.name_id.insert(name.to_string(), id);
                self.parent_children.insert(id,ids);
                if self.root.is_none() {
                    self.root = Some(id);
                }
                vec![id]
            }
            Ui::Label { name,position,text,children, style } => {
                let mut textm = self.font.text(&text,1.0);
                textm.scale(style.text_scale);
                let size = vec2(
                    textm.width() + style.text_margin * 2f32,
                    textm.height() + style.text_margin * 2f32,
                );
                let ids = children.iter().flat_map(|child|{
                    self.build(child)
                }).collect::<Vec<_>>();
                let frame = UiLabel {
                    name: name.to_string(),
                    position: *position,
                    size,
                    text: text.to_string(),
                    hover: false,
                    style: *style,
                    dragging: false,
                    drag_offset: vec2(0,0),
                    draggable: false,
                };
                let id = self.get_id();
                self.id_node.insert(id, Box::new(frame));
                self.name_id.insert(name.to_string(), id);
                self.parent_children.insert(id,ids);
                vec![id]
            }
            Ui::Image { name, position, image, style, children, size } => {
                let ids = children.iter().flat_map(|child|{
                    self.build(child)
                }).collect::<Vec<_>>();
                let image = UiImage {
                    name: name.to_string(),
                    position: *position,
                    image: *image,
                    style: *style,
                    size: *size,
                    ..Default::default()
                };
                let id = self.get_id();
                self.id_node.insert(id, Box::new(image));
                self.name_id.insert(name.to_string(), id);
                self.parent_children.insert(id,ids);
                vec![id]
            }
        }
    }
}
#[derive(Default)]
pub struct UiFrame {
    name: String,
    position:  Vec2,
    size: Vec2,
    hover: bool,
    style: UiFrameStyle,
    dragging: bool,
    drag_offset: Vec2,
    draggable: bool,
}
impl UiComponent for UiFrame {
    fn get_hover(&self) -> bool { self.hover }
    fn set_hover(&mut self, value: bool) { self.hover = value; }
    fn name(&self) -> String { self.name.to_owned() }
    fn get_position(&self) -> Vec2 { self.position }
    fn contains(&self, point: Vec2) -> bool { Rectangle::new2(self.position,self.size).contains_point(&point) }
    fn set_dragging(&mut self, value: bool) { self.dragging = value; }
    fn is_dragging(&self) -> bool { self.dragging }
    fn set_position(&mut self, value: Vec2) { self.position = value; }
    fn set_drag_offset(&mut self, value: Vec2) { self.drag_offset = value; }
    fn set_draggable(&mut self, value: bool) { self.draggable = value; }
    fn get_drag_offset(&mut self) -> Vec2 { self.drag_offset }
    fn is_draggable(&self) -> bool { self.draggable }

    #[allow(unused_variables)]
    fn draw(&self,font: &Font, mb: &mut MeshBuilder, g: &mut ShapeGfx) {
        let style = &self.style;
        if self.hover  {mb.set_style(style.hover_color)} else {mb.set_style(style.color)};
        mb.set_filled(true);
        mb.rounded_rect(self.size,8f32);
        if style.show_outline {
            mb.set_filled(false);
            if self.hover  {mb.set_style(style.hover_outline)} else {mb.set_style(style.outline)};
            mb.rounded_rect(self.size,8f32);
        }
    }
}
#[derive(Default)]
pub struct UiLabel {
    name: String,
    position:  Vec2,
    size: Vec2,
    text: String,
    hover: bool,
    style: UiLabelStyle,
    dragging: bool,
    drag_offset: Vec2,
    draggable: bool,
}
impl UiComponent for UiLabel {
    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, value: bool) {self.hover = value;}
    fn name(&self) -> String { self.name.to_owned() }
    fn get_position(&self) -> Vec2 { self.position }
    fn contains(&self, point: Vec2) -> bool {
        Rectangle::new2(self.position,self.size).contains_point(&point)
    }

    fn set_dragging(&mut self, value: bool) {
        self.dragging = value;
    }

    fn is_dragging(&self) -> bool {
        self.dragging
    }

    fn set_position(&mut self, value: Vec2) {
        self.position = value;
    }

    fn set_drag_offset(&mut self, value: Vec2) {
        self.drag_offset = value;
    }

    fn set_draggable(&mut self, value: bool) {
        self.draggable = value;
    }

    fn get_drag_offset(&mut self) -> Vec2 {
        self.drag_offset
    }

    fn is_draggable(&self) -> bool {
        self.draggable
    }

    #[allow(unused_variables)]
    fn draw(&self, font: &Font, mb: &mut MeshBuilder, g: &mut ShapeGfx) {
        let style = &self.style;
        let mut text = font.text(&self.text,1.0);
        text.uv_project();
        text.scale(style.text_scale);
        if style.show_background {
            if self.hover { mb.set_style(style.hover_color) } else { mb.set_style(style.color) };
            mb.set_filled(true);
            mb.rect(self.size);
        }
        if style.show_outline {
            mb.set_filled(false);
            if self.hover { mb.set_style(style.hover_outline) } else { mb.set_style(style.outline) };
            mb.rect(self.size);
        }
        if self.hover  {mb.set_style(style.hover_text)} else {mb.set_style(style.text)};
        let text_offset = vec2(0,text.height()) + vec2(self.style.text_margin,self.style.text_margin) / 2f32;
        mb.move_cursor(text_offset);
        mb.mesh(&text,true);
        mb.move_cursor(-text_offset);
    }
}
#[derive(Default)]
pub struct UiImage {
    hover: bool,
    name: String,
    position: Vec2,
    image: Image,
    dragging: bool,
    drag_offset: Vec2,
    draggable: bool,
    size: Option<Vec2>,
    style: UiImageStyle,
}

impl UiComponent for UiImage {
    fn get_hover(&self) -> bool { self.hover }
    fn set_hover(&mut self, value: bool) { self.hover = value; }
    fn name(&self) -> String { self.name.to_owned() }
    fn get_position(&self) -> Vec2 { self.position }
    fn contains(&self, point: Vec2) -> bool { Rectangle::new2(self.position, self.image.size()).contains_point(&point) }
    fn set_dragging(&mut self, value: bool) { self.dragging = value; }
    fn is_dragging(&self) -> bool { self.dragging }
    fn set_position(&mut self, value: Vec2) { self.position = value; }
    fn set_drag_offset(&mut self, value: Vec2) { self.drag_offset = value; }
    fn set_draggable(&mut self, value: bool) { self.draggable = value; }
    fn get_drag_offset(&mut self) -> Vec2 { self.drag_offset }
    fn is_draggable(&self) -> bool { self.draggable }

    #[allow(unused_variables)]
    fn draw(&self, font: &Font, mb: &mut MeshBuilder, g: &mut ShapeGfx) {
        g.set_tint(self.style.tint);
        let pos = mb.state.cursor;
        g.draw_mesh(&mb.build(),Vec2::ZERO);
        mb.clear_meshes();
        if let Some(size) = self.size {
            g.draw_image_sized(&self.image, size,pos);
        } else {
            g.draw_image(&self.image,pos);
        }
    }
}

pub enum Ui {
    Frame {
        name: String,
        position: Vec2,
        size: Vec2,
        children: Vec<Ui>,
        style: UiFrameStyle,
    },
    Label {
        name: String,
        position: Vec2,
        text: String,
        children: Vec<Ui>,
        style: UiLabelStyle,
    },
    Image {
        name: String,
        position: Vec2,
        image: Image,
        style: UiImageStyle,
        children: Vec<Ui>,
        size: Option<Vec2>,
    }
}
