use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem;
use std::sync::{Arc, Mutex};

use glutin;
use glutin::{ScanCode, VirtualKeyCode};
use webrender::api::*;
use winit;

use crate::gui::font;
use crate::gui::properties;

#[derive(Debug, Clone)]
pub enum PrimitiveEvent {
    Exit,
    CursorEntered,
    CursorLeft,
    CursorMoved(properties::Position),
    Button(
        properties::Position,
        properties::Button,
        properties::ButtonState,
        properties::Modifiers,
    ),
    Char(char),
    KeyInput(
        Option<VirtualKeyCode>,
        ScanCode,
        properties::ButtonState,
        properties::Modifiers,
    ),
    SetFocus(bool),
    Resized(glutin::dpi::LogicalSize),
    DPI(f64),
    HoverBegin(Vec<ItemTag>),
    HoverEnd(Vec<ItemTag>),
}

#[derive(Debug, Clone, Eq)]
pub enum ElementEvent {
    Clicked,
    FocusChange,
    HoverBegin,
    HoverEnd,
}

impl Hash for ElementEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state)
    }
}

impl PartialEq for ElementEvent {
    fn eq(&self, other: &ElementEvent) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

/*impl Copy for FnMut(&mut Element, &Any) -> bool {

}*/

//pub type EventFn = fn(&mut Element, &Any) -> bool;
pub type EventClosure = FnMut(&mut Element, &Any) -> bool;

#[derive(Clone)]
pub struct EventFn(Arc<Mutex<EventClosure>>);
//it should be safe since Element will always be within a lock.
//sending it as arc.mutex.element might end up in a deadlock
unsafe impl Send for EventFn {}
unsafe impl Sync for EventFn {}

use std::ops::DerefMut;
impl EventFn {
    pub fn new(f: Arc<Mutex<EventClosure>>) -> EventFn {
        EventFn(f)
    }

    pub fn call(&mut self, _e: &mut Element, _d: &Any) -> bool {
        //let h = ;//.unwrap()(_d)
        if let Ok(mut f) = self.0.lock() {
            let x = f.deref_mut();
            x(_e, _d)
        } else {
            false
        }
    }
}

//impl Copy for EventFn{}
//impl Copy for FnMut<&mut Element, &Any> {}

pub type EventHandlers = HashMap<ElementEvent, EventFn>;

pub trait Element: Send + Sync {
    fn get_ext_id(&self) -> u64;
    fn set(&mut self, prop: properties::Property);
    //fn get(&self, prop: &properties::Property) -> Option<&properties::Property>;
    fn get_properties(&self) -> properties::Properties;
    fn render(
        &mut self,
        api: &RenderApi,
        builder: &mut DisplayListBuilder,
        extent: properties::Extent,
        font_store: &mut font::FontStore,
        props: Option<Arc<properties::Properties>>,
        id: &mut properties::IdGenerator
    );
    fn get_bounds(&self) -> properties::Extent;
    #[allow(unused)]
    fn on_primitive_event(&mut self, item_tag: &[ItemTag], e: PrimitiveEvent) -> bool;
    #[allow(unused)]
    fn set_handler(&mut self, e: ElementEvent, f: EventFn) {}
    #[allow(unused)]
    fn exec_handler(&mut self, e: ElementEvent, d: &Any) -> bool {
        false
    }
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
    #[allow(unused)]
    fn on_event(
        &mut self,
        event: winit::WindowEvent,
        api: &RenderApi,
        document_id: DocumentId,
    ) -> bool {
        false
    }
}

pub type ElementObj = Arc<Mutex<Element>>;

pub trait HasChildren: Element {
    #[allow(unused)]
    fn get_child(&self, i: u32) -> Option<Arc<Mutex<Element>>> {
        None
    }
    /*#[allow(unused)]
    fn get_child_mut(&mut self, i:u32) -> Option<Arc> {None}*/
    #[allow(unused)]
    fn append(&mut self, e: Arc<Mutex<Element>>) -> Option<Arc<Mutex<Element>>> {
        None
    }
}

pub trait CanDisable: Element {
    fn set_enabled(&mut self, _: bool);
    fn get_enabled(&self) -> bool;
}
