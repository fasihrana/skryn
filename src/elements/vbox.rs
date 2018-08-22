use std::sync::Arc;
use std::any::Any;

use webrender::api::*;

use util::*;
use elements::element::*;
use gui::properties;
use gui::font;

pub struct VBox {
    ext_id: u64,
    children: Vec<Box<Element>>,
    props: properties::Properties,
    bounds: properties::Extent,
    handlers: EventHandlers,
}

impl VBox {
    pub fn new() -> Self{
        let mut props = properties::Properties::new();
        props.default();
        VBox {
            ext_id:0,
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

impl Element for VBox {
    fn get_ext_id(&self)->u64{self.ext_id}

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
              _props: Option<Arc<properties::Properties>>,
              gen: &mut properties::IdGenerator) {

        let bgcolor = self.props.get_bg_color();

        let _id = gen.get();

        let mut info = LayoutPrimitiveInfo::new((self.bounds.x, self.bounds.y).by(self.bounds.w, self.bounds.h));
        info.tag = Some((_id, 0));
        builder.push_rect(&info, bgcolor);

        let next_x = 0.0;
        let mut next_y = 0.0;

        for elm in self.children.iter_mut(){
            let child_extent = properties::Extent{
                x: next_x + extent.x,
                y: next_y + extent.y,
                w: extent.w,
                h: extent.h,
                dpi: extent.dpi,
            };

            elm.render(builder,child_extent,font_store,None,gen);
            let _ex = elm.get_bounds();
            next_y += _ex.h;
        }

        //only her for debugging.
        if next_y == 0.0 {
            next_y = extent.h;
        }

        self.bounds = properties::Extent{
            x: extent.x,
            y: extent.y,
            w: extent.w,
            h: next_y,
            dpi: extent.dpi,
        };

        println!("{:?}", self.bounds);
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, e: PrimitiveEvent) -> bool {
        let mut handled = false;
        for elm in self.children.iter_mut() {
            match e.clone() {
                PrimitiveEvent::Button(p,_b,_s,_m) => {
                    if !handled {
                        let _b = elm.get_bounds();
                        if p.x >= _b.x && p.x <= (_b.w + _b.x)
                            && p.y >= _b.y && p.y <= (_b.h + _b.y) {
                            handled = elm.on_primitive_event(e.clone());
                        }
                    }
                },
                PrimitiveEvent::Char(_c) => {
                    handled = elm.on_primitive_event(e.clone());
                },
                PrimitiveEvent::SetFocus(_f,p) => {
                    if let Some(p) = p {
                        let _b = elm.get_bounds();
                        if p.x >= _b.x && p.x <= (_b.w + _b.x)
                            && p.y >= _b.y && p.y <= (_b.h + _b.y) {
                            handled = elm.on_primitive_event(e.clone());
                        } else {
                            handled = elm.on_primitive_event(PrimitiveEvent::SetFocus(false, None));
                        }
                    } else {
                        handled = elm.on_primitive_event(PrimitiveEvent::SetFocus(false, None));
                    }
                },
                _ => ()
            }
        }
        if !handled {
            match e.clone() {
                PrimitiveEvent::Button(_p,_b,_s,_m) => {
                    if _s == properties::ButtonState::Released {
                        let handler = self.get_handler(ElementEvent::Clicked);
                        handled = handler(self, &_m);
                    }
                },
                _ => ()
            }
        }
        return handled;
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

impl HasChildren for VBox {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    #[allow(unused)]
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    fn append(&mut self, e:Box<Element>) -> Option<Box<Element>>{
        self.children.push(e);
        None
    }

}