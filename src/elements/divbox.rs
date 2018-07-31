use std::sync::Arc;

use webrender::api::*;

use elements::element::*;
use gui::properties;
use gui::font;
//use elements::textbox::TextBox;

pub struct DivBox{
    children: Vec<Box<Element>>,
    props: properties::Properties,
    bounds: properties::Extent,
}

impl DivBox{
    pub fn new() -> Self{
        let mut props = properties::Properties::new();
        props.default();
        DivBox{
            children:Vec::new(),
            props,
            bounds: properties::Extent{
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0,
            }
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
              props: Option<Arc<properties::Properties>>) {

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

        let next_x = extent.x;
        let mut next_y = extent.y;

        for elm in self.children.iter_mut() {
            elm.render(builder,
                                        properties::Extent{
                                            x:next_x,
                                            y:next_y,
                                            w:extent.w,
                                            h:extent.h,
                                            dpi: extent.dpi,
                                        },
                                        font_store,
                                        props.clone());

            let tmp_extent = elm.get_bounds();

            //next_x = tmp_extent.w;
            next_y = tmp_extent.h;
        }

        builder.pop_stacking_context();

        self.bounds = properties::Extent{
            x: extent.x,
            y: extent.y,
            w: extent.w,
            h: extent.h,
            dpi: extent.dpi,
        };
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_event(&mut self, e: PrimitiveEvent) {
        for elm in self.children.iter_mut() {
            match e.clone() {
                PrimitiveEvent::Button(p,b,s,m) =>{
                    let _b = elm.get_bounds();
                    if p.x >= _b.x && p.x <= (_b.w + _b.x)
                        && p.y >= _b.y && p.y <= (_b.h + _b.y) {
                        elm.on_event(e.clone());
                    }
                },
                PrimitiveEvent::Char(c) => {
                    elm.on_event(e.clone());
                },
                PrimitiveEvent::SetFocus(f,p) => {
                    if let Some(p) = p {
                        let _b = elm.get_bounds();
                        if p.x >= _b.x && p.x <= (_b.w + _b.x)
                            && p.y >= _b.y && p.y <= (_b.h + _b.y) {
                            elm.on_event(e.clone());
                        } else {
                            elm.on_event(PrimitiveEvent::SetFocus(false, None));
                        }
                    } else {
                        elm.on_event(PrimitiveEvent::SetFocus(false, None));
                    }
                },
                _ => ()
            }
        }
    }
}

impl HasChildren for DivBox {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    #[allow(unused)]
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    fn append(&mut self, e:Box<Element>) {
        self.children.push(e);
    }
}