use std::sync::Arc;
use std::any::Any;
use std::mem;

use webrender::api::*;

use util::*;
use elements::element::*;
use gui::properties;
use gui::font;

pub struct ScrollBox {
    ext_id: u64,
    child: Option<Box<Element>>,
    props: properties::Properties,
    bounds: properties::Extent,
    content: properties::Extent,
    handlers: EventHandlers,
}

impl ScrollBox {
    pub fn new() -> Self{
        let mut props = properties::Properties::new();
        props.default();
        ScrollBox {
            ext_id:0,
            child: None,
            props,
            bounds: properties::Extent{
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0,
            },
            content: properties::Extent{
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

impl Element for ScrollBox {
    fn get_ext_id(&self)->u64{self.ext_id}

    fn render(&mut self,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              _props: Option<Arc<properties::Properties>>,
              gen: &mut properties::IdGenerator) {

        let bgcolor = self.props.get_bg_color();

        let _id = gen.get();
        self.ext_id = _id;

        let mut bounds = properties::Extent{
            x:  0.0,
            y:  0.0,
            w:  0.0,
            h:  0.0,
            dpi:0.0
        };

        self.bounds = extent.clone();

        builder.push_stacking_context(
            &LayoutPrimitiveInfo::new((extent.x, extent.y).by(0.0, 0.0)),
            None,
            TransformStyle::Flat,
            MixBlendMode::Normal,
            Vec::new(),
            GlyphRasterSpace::Screen,
        );

        let pipeline_id = builder.pipeline_id.clone();
        let scroll_frame = builder.define_scroll_frame(Some(ExternalScrollId(_id,pipeline_id)),
                                    (self.content.x,self.content.y).by(self.content.w, self.content.h),
                                    (0.0,0.0).by(extent.w,extent.h),
                                    vec![],
                                    None,
                                    ScrollSensitivity::ScriptAndInputEvents);
        builder.push_clip_id(scroll_frame);

        let mut info = LayoutPrimitiveInfo::new((self.content.x,self.content.y).by(self.content.w, self.content.h));
        info.tag = Some((_id, 0));
        builder.push_rect(&info, bgcolor);

        if let Some(ref mut elm) = self.child {
            elm.render(builder,extent.clone(),font_store,None,gen);
            bounds = elm.get_bounds();
        }

        builder.pop_clip_id(); //scroll frame
        builder.pop_stacking_context();

        self.content = bounds;
    }

    fn set(&mut self, prop: properties::Property) {
        self.props.set(prop);
    }

    fn get(&self, prop: &properties::Property) -> Option<&properties::Property> {
        self.props.get(&prop)
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, e: PrimitiveEvent) -> bool {
        let mut handled = false;
        if let Some(ref mut elm)  = self.child {
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

impl HasChildren for ScrollBox {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    #[allow(unused)]
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    fn append(&mut self, e:Box<Element>) -> Option<Box<Element>> {
        let mut ret = Some(e);
        mem::swap(&mut self.child,&mut ret);
        return ret;
    }

}