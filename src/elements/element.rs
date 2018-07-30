use std::sync::Arc;

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
}

pub trait Element {
    fn set(&mut self, prop: properties::Property);
    fn get(&self, prop: &properties::Property) -> Option<&properties::Property>;
    fn render(&mut self,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              props: Option<Arc<properties::Properties>>);
    fn get_bounds(&self) -> properties::Extent;
    fn on_event(&mut self, e: PrimitiveEvent);
}


pub trait HasChildren {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    #[allow(unused)]
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    #[allow(unused)]
    fn append(&mut self, e:Box<Element>) {}
}