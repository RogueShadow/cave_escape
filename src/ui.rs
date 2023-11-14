use neo_granseal::prelude::*;
use neo_granseal::util::Rectangle;
use crate::cave::CaveEvent;

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
