use std::sync::{Arc,Mutex};
use std::any::Any;
use std::mem;

use webrender::api::*;

use util::*;
use elements::element::*;
use gui::properties;
use gui::font;

pub struct ScrollBox {
    ext_id: u64,
    child: Option<Arc<Mutex<Element>>>,
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
            match elm.lock() {
                Ok(ref mut elm) => {
                    elm.render(builder,extent.clone(),font_store,None,gen);
                    bounds = elm.get_bounds();
                },
                Err(_err_str) => panic!("unable to lock element : {}",_err_str)
            }
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

    fn on_primitive_event(&mut self, ext_ids:&[ItemTag], e: PrimitiveEvent) -> bool {
        let mut handled = false;
        if let Some (ref mut _child_elm) = self.child {
            match (&e,_child_elm.lock()) {
                (PrimitiveEvent::SetFocus(_), Ok(ref mut _child_elm)) => {
                    if ext_ids.len() > 1
                        && ext_ids[0].0 == self.ext_id
                        && ext_ids[1].0 == _child_elm.get_ext_id() {
                        _child_elm.on_primitive_event(&ext_ids[1..], PrimitiveEvent::SetFocus(true));
                    } else {
                        _child_elm.on_primitive_event(&[], PrimitiveEvent::SetFocus(false));
                    }
                },
                (PrimitiveEvent::Char(_c), Ok(ref mut _child_elm)) => {
                    handled = _child_elm.on_primitive_event(&[],e.clone());
                },
                (_, Ok(ref mut _child_elm)) =>  {
                    if !handled {
                        if ext_ids.len() == 1 {
                            handled = _child_elm.on_primitive_event(&[], e.clone());
                        } else if ext_ids.len() > 1 {
                            handled = _child_elm.on_primitive_event(&ext_ids[1..], e.clone());
                        }
                    }
                },
                (_,Err(_err_str)) => {
                    //this should be unreachable
                    panic!("unable to lock element : {}", _err_str)
                }
            }
        }
        // if none of the children handled the event
        // see if you can handle it here
        if !handled {
            match e {
                PrimitiveEvent::Button(_p,b,s,m) => {
                    let handler = self.get_handler(ElementEvent::Clicked);
                    handled = handler(self, &m);
                },
                _ => ()
            }
        }
        return handled;
    }

    fn set_handler(&mut self, _e: ElementEvent, _f:EventFn) {
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

    fn as_any(&self) -> &Any{
        self
    }
    fn as_any_mut(&mut self) -> &mut Any{
        self
    }
}

impl HasChildren for ScrollBox {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<Arc<Mutex<Element>>> {None}
    #[allow(unused)]
    //fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    fn append(&mut self, e:Arc<Mutex<Element>>) -> Option<Arc<Mutex<Element>>> {
        let mut ret = Some(e);
        mem::swap(&mut self.child,&mut ret);
        return ret;
    }

}