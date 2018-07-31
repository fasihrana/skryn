use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::mem;
use std::any::Any;

use webrender::api::*;

use gui::font;
use gui::properties;

#[derive(Debug, Clone)]
pub enum PrimitiveEvent {
    Exit,
    CursorEntered,
    CursorLeft,
    CursorMoved(properties::Position),
    Button(properties::Position, properties::Button, properties::ButtonState, properties::Modifiers),
    Char(char),
    SetFocus(bool,Option<properties::Position>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ElementEvent{
    Clicked,
    FocusChange,
}

impl Hash for ElementEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state)
    }
}

pub type EventFn = fn(&mut Element, &Any);

pub fn default_fn(_e:&mut Element, _d: &Any){}

pub trait Element {
    fn set(&mut self, prop: properties::Property);
    fn get(&self, prop: &properties::Property) -> Option<&properties::Property>;
    fn render(&mut self,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              props: Option<Arc<properties::Properties>>);
    fn get_bounds(&self) -> properties::Extent;
    fn on_primitive_event(&mut self, e: PrimitiveEvent);
    fn set_event(&mut self, _e: ElementEvent, _f:EventFn){}
    fn as_any(&self) -> &Any;
}

pub trait HasChildren : Element {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    #[allow(unused)]
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    #[allow(unused)]
    fn append(&mut self, e:Box<Element>) {}
}




