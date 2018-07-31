use std::sync::Arc;
use std::any::Any;

use webrender::api::*;

use elements::element::*;
use gui::properties;
use gui::font;

pub struct DivBox{
    children: Vec<Box<Element>>,
    props: properties::Properties,
    bounds: properties::Extent,
    handlers: EventHandlers,
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
            },
            handlers: EventHandlers::new(),
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

    fn on_primitive_event(&mut self, e: PrimitiveEvent) {
        let mut handled_click = false;
        for elm in self.children.iter_mut() {
            match e.clone() {
                PrimitiveEvent::Button(p,_b,_s,_m) =>{
                    let _b = elm.get_bounds();
                    if p.x >= _b.x && p.x <= (_b.w + _b.x)
                        && p.y >= _b.y && p.y <= (_b.h + _b.y) {
                        elm.on_primitive_event(e.clone());
                        handled_click = true;
                    }
                },
                PrimitiveEvent::Char(_c) => {
                    elm.on_primitive_event(e.clone());
                },
                PrimitiveEvent::SetFocus(_f,p) => {
                    if let Some(p) = p {
                        let _b = elm.get_bounds();
                        if p.x >= _b.x && p.x <= (_b.w + _b.x)
                            && p.y >= _b.y && p.y <= (_b.h + _b.y) {
                            elm.on_primitive_event(e.clone());
                        } else {
                            elm.on_primitive_event(PrimitiveEvent::SetFocus(false, None));
                        }
                    } else {
                        elm.on_primitive_event(PrimitiveEvent::SetFocus(false, None));
                    }
                },
                _ => ()
            }
        }
        if !handled_click {
            match e.clone() {
                PrimitiveEvent::Button(_p,_b,_s,_m) => {
                    if _s == properties::ButtonState::Released {
                        let handler = self.get_handler(ElementEvent::Clicked);
                        handler(self, &_p);
                    }
                },
                _ => ()
            }
        }
    }

    fn set_handler(&mut self, _e: ElementEvent, _f: EventFn) {
        self.handlers.insert(_e, _f);
    }

    fn get_handler(&mut self, _e: ElementEvent) -> EventFn {
        let eh = &mut self.handlers;
        let h = eh.get(&_e);
        if let Some(h) = h{
            h.clone()
        } else {
            default_fn
        }
    }

    fn as_any(&self) -> &Any {
        self
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