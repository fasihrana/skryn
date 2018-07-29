use std::sync::Arc;

use webrender::api::*;

use gui::font;
use gui::properties;

pub trait Element {
    fn set(&mut self, prop: properties::Property);
    fn get(&self, prop: &properties::Property) -> Option<&properties::Property>;
    fn render(&mut self,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              props: Option<Arc<properties::Properties>>) -> properties::Extent;
}

pub trait HasChildren {
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    fn append(&mut self, e:Box<Element>) {}
}

/*pub enum ElementType <E:Element, EC:Element+HasChildren> {
    E(E),
    EC(EC),
}*/