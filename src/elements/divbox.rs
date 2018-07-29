use std::sync::Arc;

use webrender::api::*;

use elements::element::*;
use gui::properties;
use gui::font;
//use elements::textbox::TextBox;

pub struct DivBox{
    children: Vec<Box<Element>>,
    props: properties::Properties,
}

impl DivBox{
    pub fn new() -> Self{
        let mut props = properties::Properties::new();
        props.default();
        DivBox{
            children:Vec::new(),
            props,
        }
    }
}

impl Element for DivBox {
    fn set(&mut self, prop: properties::Property) {
        self.props.set(prop);
    }

    fn get(&self, prop: &properties::Property) -> Option<&properties::Property> {
        self.props.get(&prop)
    }

    fn render(&mut self,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              props: Option<Arc<properties::Properties>>) -> properties::Extent {

        let bgcolor = self.props.get_bg_color();

        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(extent.x,extent.y),
            LayoutSize::new(extent.w,extent.h)
        ));

        builder.push_stacking_context(
            &info,
            None,
            TransformStyle::Flat,
            MixBlendMode::Normal,
            Vec::new(),
            GlyphRasterSpace::Screen,
        );

        builder.push_rect(&info, bgcolor);

        let mut next_x = extent.x;
        let mut next_y = extent.y;

        for elm in self.children.iter_mut() {
            let tmp_extent = elm.render(builder,
                                        properties::Extent{
                                            x:next_x,
                                            y:next_y,
                                            w:extent.w,
                                            h:extent.h,
                                            dpi: extent.dpi,
                                        },
                                        font_store,
                                        props.clone());

            //next_x = tmp_extent.w;
            next_y = tmp_extent.h;
        }

        builder.pop_stacking_context();

        properties::Extent{
            x: extent.x,
            y: extent.y,
            w: extent.w,
            h: extent.h,
            dpi: extent.dpi,
        }
    }
}

impl HasChildren for DivBox {
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    fn append(&mut self, e:Box<Element>) {
        self.children.push(e);
    }
}