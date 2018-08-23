use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::mem;
use std::collections::HashMap;
use std::any::Any;

use winit;
use webrender::api::*;

use gui::font;
use gui::properties;

#[derive(Debug, Clone)]
pub enum PrimitiveEvent {
    /*Exit,
    CursorEntered,
    CursorLeft,
    CursorMoved(properties::Position),*/
    Button(properties::Position, properties::Button, properties::ButtonState, properties::Modifiers),
    Char(char),
    SetFocus(bool),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ElementEvent{
    Clicked,
    FocusChange,
    //Char,
}

impl Hash for ElementEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state)
    }
}

pub type EventFn = /*FnMut(&mut Element, &Any) -> bool;/ */fn(&mut Element, &Any) -> bool;

pub type EventHandlers = HashMap<ElementEvent,EventFn>;

pub fn default_fn(_e:&mut Element, _d: &Any)->bool{false}

pub trait Element {
    fn get_ext_id(&self) -> u64;
    fn set(&mut self, prop: properties::Property);
    fn get(&self, prop: &properties::Property) -> Option<&properties::Property>;
    fn render(&mut self,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              props: Option<Arc<properties::Properties>>,
              &mut properties::IdGenerator);
    fn get_bounds(&self) -> properties::Extent;
    fn on_primitive_event(&mut self, &[ItemTag], e: PrimitiveEvent) -> bool;
    fn set_handler(&mut self, _e: ElementEvent, _f:EventFn){}
    fn get_handler(&mut self, _e: ElementEvent) -> EventFn{ default_fn }
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
    #[allow(unused)]
    fn on_event(&mut self, event: winit::WindowEvent, api: &RenderApi, document_id: DocumentId) -> bool {false}
}

pub trait HasChildren : Element {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    #[allow(unused)]
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    #[allow(unused)]
    fn append(&mut self, e:Box<Element>) -> Option<Box<Element>>{None}
}




