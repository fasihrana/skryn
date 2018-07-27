use std::sync::Arc;

use webrender::api::*;

use gui::font;
use gui::properties;

pub trait Element{
    fn set(&mut self, prop: properties::Property);
    fn get(&self, prop: &properties::Property) -> Option<&properties::Property>;
    fn render(&mut self,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              props: Option<Arc<properties::Properties>>);
}

pub trait HasChildren{
    fn get(&self, i:u32) -> Option<&Element>;
    fn get_mut(&mut self, i:u32) -> Option<&mut Element>;
    fn append(&mut self, e:impl Element);
}