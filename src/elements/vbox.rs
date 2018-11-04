use std::sync::{Arc,Mutex};
use std::any::Any;

use webrender::api::*;

use util::*;
use elements::element::*;
use gui::properties;
use gui::font;

pub struct VBox {
    ext_id: u64,
    children: Vec<Arc<Mutex<Element>>>,
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

    fn get_height_sums(&mut self) -> (f32,f32) {
        let top = self.props.get_top();
        let bottom = self.props.get_bottom();

        let mut stretchy:f32 = 0.0;
        let mut pixel:f32 = 0.0;

        match top {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => ()
        }

        match bottom {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => ()
        }

        for elm in self.children.iter_mut() {
            if let Ok(_e) = elm.lock() {
                let _h = _e.get(&properties::HEIGHT).unwrap().clone();
                match _h {
                    properties::Property::Height(properties::Unit::Stretch(_s)) => stretchy += _s,
                    _ => {
                        let t_b = _e.get_bounds();
                        pixel += t_b.h;
                    }
                }
            }
        }

        (pixel,stretchy)
    }

    fn get_width_sums(&mut self) -> (f32,f32) {
        let left = self.props.get_left();
        let right = self.props.get_right();
        let width = self.props.get_width();

        let mut stretchy:f32 = 0.0;
        let mut pixel:f32 = 0.0;

        match left {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => ()
        }

        match right {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => ()
        }

        match width {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => ()
        }

        (pixel,stretchy)
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
              api: &RenderApi,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              _props: Option<Arc<properties::Properties>>,
              gen: &mut properties::IdGenerator) {

        let bgcolor = self.props.get_bg_color();
        let top = self.props.get_top();
        let bottom = self.props.get_bottom();
        let left = self.props.get_left();
        let right = self.props.get_right();
        let width = self.props.get_width();
        let height = self.props.get_height();

        let (wp_sum, ws_sum )= self.get_width_sums();
        let mut remaining_width = extent.w - wp_sum;
        if remaining_width < 0.0 {remaining_width = 0.0;}
        let mut w_stretchy_factor = remaining_width / ws_sum;
        if w_stretchy_factor.is_nan() {
            w_stretchy_factor = 0.0;
        }

        let (hp_sum, hs_sum )= self.get_height_sums();
        let mut remaining_height = extent.h - hp_sum;
        if remaining_height < 0.0 {remaining_height = 0.0;}
        let mut h_stretchy_factor = remaining_height / hs_sum;
        if h_stretchy_factor.is_nan() {
            h_stretchy_factor = 0.0;
        }

        let _id = gen.get();
        self.ext_id = _id;

        let mut info = LayoutPrimitiveInfo::new((self.bounds.x, self.bounds.y).by(self.bounds.w, self.bounds.h));
        info.tag = Some((_id, 0));
        builder.push_rect(&info, bgcolor);

        let mut next_x = 0.0;
        let mut next_y = 0.0;

        match top {
            properties::Unit::Stretch(_s) => next_y = h_stretchy_factor * _s,
            properties::Unit::Pixel(_p) => next_y = _p,
            _ => (),
        }

        match left {
            properties::Unit::Stretch(_s) => next_x = w_stretchy_factor * _s,
            properties::Unit::Pixel(_p) => next_x = _p,
            _ => (),
        }

        match height {
            properties::Unit::Stretch(_s) => remaining_height = _s * h_stretchy_factor,
            properties::Unit::Pixel(_p) => remaining_height = _p,
            _ => ()
        }

        match width {
            properties::Unit::Stretch(_s) => remaining_width = _s * w_stretchy_factor,
            properties::Unit::Pixel(_p) => remaining_width = _p,
            _ => ()
        }

        for elm in self.children.iter_mut(){
            let mut child_extent = properties::Extent{
                x: next_x + extent.x,
                y: next_y + extent.y,
                w: remaining_width,
                h: remaining_height,
                dpi: extent.dpi,
            };

            match elm.lock() {
                Ok(ref mut elm) => {
                    let e_height = elm.get(&properties::HEIGHT).unwrap().clone();

                    match e_height {
                        properties::Property::Height(properties::Unit::Pixel(_p)) => {
                            child_extent.h = _p;
                        },
                        properties::Property::Height(properties::Unit::Stretch(_s)) => {
                            child_extent.h = h_stretchy_factor;
                        },
                        _ => ()
                    }

                    elm.render(api, builder, child_extent, font_store, None, gen);
                    let _ex = elm.get_bounds();
                    next_y += _ex.h;
                },
                Err(_err_str) => panic!("unable to lock element : {}",_err_str)
            }
        }

        match bottom {
            properties::Unit::Stretch(_s) => next_y += h_stretchy_factor * _s,
            properties::Unit::Pixel(_p) => next_y += _p,
            _ => (),
        }

        next_x += remaining_width;
        match right {
            properties::Unit::Stretch(_s) => next_x +=  w_stretchy_factor * _s,
            properties::Unit::Pixel(_p) => next_x += _p,
            _ => (),
        }

        // TODO: Remove
        // only here for debugging.
        if next_y == 0.0 {
            next_y = extent.h;
        }

        self.bounds = properties::Extent{
            x: extent.x,
            y: extent.y,
            w: next_x,
            h: next_y,
            dpi: extent.dpi,
        };
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, ext_ids:&[ItemTag], e: PrimitiveEvent) -> bool {
        let mut handled = false;
        for _child_elm in self.children.iter_mut() {
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
                    if handled {
                        break;
                    }
                },
                (PrimitiveEvent::HoverBegin(_), Ok(ref mut _child_elm)) =>  {
                    _child_elm.on_primitive_event(&[],e.clone());
                },
                (PrimitiveEvent::HoverEnd(_), Ok(ref mut _child_elm)) =>  {
                    _child_elm.on_primitive_event(&[],e.clone());
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
                PrimitiveEvent::Button(_p,_b,_s,m) => {
                    handled = self.exec_handler(ElementEvent::Clicked, &m);
                },
                _ => ()
            }
        }
        return handled;
    }

    fn set_handler(&mut self, _e: ElementEvent, _f:EventFn) {
        self.handlers.insert(_e, _f);
    }

    fn exec_handler(&mut self, _e: ElementEvent, _d: &Any) -> bool {
        let eh = &mut self.handlers;
        let h = eh.get_mut(&_e);
        if let Some(h) = h{
            (h.0)(_d)
        } else {
            false
        }
    }

    fn as_any(&self) -> &Any{
        self
    }
    fn as_any_mut(&mut self) -> &mut Any{
        self
    }
}

impl HasChildren for VBox {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<Arc<Mutex<Element>>> {None}
    #[allow(unused)]
    //fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    fn append(&mut self, e:Arc<Mutex<Element>>) -> Option<Arc<Mutex<Element>>>{
        self.children.push(e);
        None
    }

}
