use std::sync::Arc;
use std::any::Any;
use std::collections::{HashSet};
use std::iter::FromIterator;

use glutin::VirtualKeyCode;
use webrender::api::*;
use clipboard::{ClipboardProvider, ClipboardContext};

use elements::element::*;
use gui::properties;
use gui::font;

pub struct TextBox {
    ext_id: u64,
    value: String,
    props: properties::Properties,
    bounds: properties::Extent,
    cache: Vec<((f32,f32),(f32,f32))>,
    focus: bool,
    event_handlers: EventHandlers,
    drawn: u8,
    editable: bool,
    enabled: bool,
    singleline: bool,
    cursor: usize,
    hovering: bool,
    is_password: bool,
}

impl TextBox {
    pub fn new(s: String) -> Self {
        let mut props = properties::Properties::new();
        props.default();
        props.set(properties::Property::Height(properties::Unit::Natural));
        TextBox {
            ext_id: 0,
            value: s,
            props,
            bounds: properties::Extent {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0,
            },
            cache: vec![],
            //char_ext: vec![],
            focus: false,
            event_handlers: EventHandlers::new(),
            drawn: 0,
            editable: true,
            enabled: true,
            singleline: false,
            cursor: 0,
            hovering: false,
            is_password: false,
        }
    }

    pub fn set_value(&mut self, s: String) {
        self.value = s;
        //self.cache.clear();
        //self.char_ext.clear();
        self.drawn = 0;
    }

    pub fn append_value(&mut self, s: String) {
        self.value = format!("{}{}",self.value,s);
        //self.cache.clear();
        //self.char_ext.clear();
        self.drawn = 0;
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }

    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
        self.drawn = 0;
        if !editable {
            self.focus = false;
        }
    }

    pub fn get_editable(&self) -> bool {
        self.editable
    }

    pub fn set_is_password(&mut self, val: bool){
        self.is_password = val;
    }

    pub fn set_singleline(&mut self, singleline: bool) {
        self.singleline = singleline;
    }

    pub fn get_index_at(&self, p: properties::Position) -> usize {
        let size = self.props.get_size() as f32;
        let mut i = 0;
        let mut cursor = 0;
        while i < self.cache.len() {
            let pmin = self.cache[i as usize].0;
            let pmax = self.cache[i as usize].1;
            if p.y > (pmax.1 - size) && p.x > pmin.0 {
                //if the click is before half-x of the character, assign i
                //otherwise assign i+1
                let mid_x = ((pmax.0 - pmin.0)/2.0) + pmin.0;
                if p.x <  mid_x {
                    cursor = i;
                } else {
                    cursor = i+1;
                }
            }
            i += 1;
        }

        cursor
    }
}

impl Element for TextBox {
    fn get_ext_id(&self) -> u64 { self.ext_id }

    fn set(&mut self, prop: properties::Property) {
        self.props.set(prop);
    }

    /*fn get(&self, prop: &properties::Property) -> Option<&properties::Property> {
        self.props.get(&prop)
    }*/

    fn get_properties(&self) -> properties::Properties {
        self.props.clone()
    }

    fn render(&mut self,
              _api: &RenderApi,
              builder: &mut DisplayListBuilder,
              extent: properties::Extent,
              font_store: &mut font::FontStore,
              _props: Option<Arc<properties::Properties>>,
              gen: &mut properties::IdGenerator) {

        let _id = gen.get();
        self.ext_id = _id;

        let (mut cursor_x, mut cursor_y, cursor_i) = (0.0, 0.0, self.cursor);

        let mut cache = vec![];
        let mut glyphs = vec![];
        let size = self.props.get_size() as f32;
        let family = self.props.get_family();
        let mut color = self.props.get_color();
        let mut bgcolor = self.props.get_bg_color();
        let width = self.props.get_width();
        let height = self.props.get_height();

        if self.hovering {
            color = self.props.get_hover_color();
            bgcolor = self.props.get_hover_bg_color();
        }

        if self.focus && self.editable {
            color = self.props.get_focus_color();
            bgcolor = self.props.get_focus_bg_color();
        }

        if !self.enabled {
            color = self.props.get_disabled_color();
            bgcolor = self.props.get_disabled_bg_color();
        }

        let (f_key, fi_key) = font_store.get_font_instance(&family, size as i32);

        let mut next_x = extent.x;
        let mut next_y = extent.y + size;

        let val_str = "●".repeat(self.value.len());
        let char_set: HashSet<char> = if self.is_password {
            HashSet::from_iter(val_str.chars())
        }
        else {
            HashSet::from_iter(self.value.chars())
        };

        let mappings = font_store.get_glyphs(f_key, fi_key, &char_set);

        let val_str = if self.is_password {
            val_str.chars()
        } else {
            self.value.chars()
        };

        let mut text_iter = val_str;//self.value.chars();

        let mut max_x: f32 = 0.0;

        let mut c_indx = 0;

        let mut skip;

        loop {
            skip = false;
            let _char = text_iter.next();
            if _char.is_none() {
                break;
            }
            let _char = _char.unwrap();

            if !self.singleline && (_char == '\r' || _char == '\n') {
                next_y = next_y + size;
                next_x = extent.x;
                skip = true;
            }
            if _char == ' ' || _char == '\t' {
                next_x += size / 3.0;
            }

            let _glyph = mappings.get(&_char);

            let (start_x, start_y) = (next_x, next_y);

            if let Some((gi, gd)) = _glyph {
                if !skip {
                    glyphs.push(GlyphInstance {
                        index: gi.to_owned(),
                        point: LayoutPoint::new(next_x, next_y),
                    });

                    if c_indx == cursor_i {
                        cursor_x = next_x;
                        cursor_y = next_y;
                    }

                    next_x = next_x + gd.advance;
                }
            }

            if max_x < next_x {
                max_x = next_x;
            }

            cache.push(((start_x,start_y),(next_x,next_y)));

            c_indx += 1;

            if c_indx == cursor_i {
                cursor_x = next_x;
                cursor_y = next_y;
            }
        }

        self.cache = cache;

        let mut calc_w = max_x - extent.x;
        let mut calc_h = next_y - extent.y;

        calc_w = match width {
            properties::Unit::Extent => extent.w,
            properties::Unit::Pixel(px) => px,
            properties::Unit::Stretch(s) => s * extent.w,
            properties::Unit::Natural => calc_w,
        };

        calc_h = match height {
            properties::Unit::Extent => extent.h,
            properties::Unit::Pixel(px) => px,
            properties::Unit::Stretch(s) => s * extent.h,
            properties::Unit::Natural => calc_h,
        };

        self.bounds = properties::Extent {
            x: extent.x,
            y: extent.y,
            w: calc_w,
            h: calc_h,
            dpi: extent.dpi,
        };

        let mut info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(extent.x, extent.y),
            LayoutSize::new(self.bounds.w, self.bounds.h),
        ));
        info.tag = Some((_id, 0));
        builder.push_rect(&info, bgcolor);

        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(extent.x, extent.y),
            LayoutSize::new(self.bounds.w, self.bounds.h),
        ));
        builder.push_text(&info,
                          &glyphs,
                          fi_key.clone(),
                          color.clone(),
                          Some(GlyphOptions::default()));

        //add the cursor
        if self.focus && self.enabled && self.editable {
            let info = LayoutPrimitiveInfo::new(LayoutRect::new(
                LayoutPoint::new(cursor_x, cursor_y - size),
                LayoutSize::new(1.0, size),
            ));
            builder.push_rect(&info, color.clone());
        }
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, ext_ids: &[ItemTag], e: PrimitiveEvent) -> bool {
        let mut handled = false;
        match e {
            PrimitiveEvent::Char(c) => {
                if self.focus && self.enabled && self.editable {
                    if c == '\x08' {
                        let mut l = self.cursor;
                        if l > 0 {
                            l = l - 1;
                            while !self.value.is_char_boundary(l) && l > 0 {
                                l = l - 1;
                            }
                            self.value = format!("{}{}", &self.value[0..l], &self.value[self.cursor..]);
                            self.cursor = l;
                        }
                    } else if c == '\u{3}' {
                        if !self.is_password {
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            ctx.set_contents(self.value.clone()).unwrap();
                        }
                    } else if c == '\u{16}' {
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        let vstr = ctx.get_contents().unwrap();
                        self.value = format!("{}{}{}", &self.value[0..self.cursor], &vstr[0..], &self.value[self.cursor..]);
                        self.cursor += vstr.len();
                    } else {
                        //self.value.push(c);
                        //
                        if self.cursor == 0 {
                            let mut newstr = format!("{}", c);
                            newstr.push_str(&self.value[0..]);
                            self.value = newstr;
                        } else if self.cursor < self.value.len() {
                            self.value = format!("{}{}{}", &self.value[0..self.cursor], c, &self.value[self.cursor..]);
                            ;
                        } else {
                            self.value.push(c);
                        }
                        self.cursor += 1;
                    }
                    handled = true;
                }
            },
            PrimitiveEvent::Button(_p, b, s, m) => {
                if ext_ids.len() > 0 && ext_ids[0].0 == self.ext_id
                    && b == properties::Button::Left
                    && s == properties::ButtonState::Released
                    {
                        self.cursor = self.get_index_at(_p.clone());
                        handled = self.exec_handler(ElementEvent::Clicked, &m);
                    }
            },
            PrimitiveEvent::SetFocus(f) => {
                if self.enabled {
                    if self.focus != f {
                        self.focus = f;
                        handled = self.exec_handler(ElementEvent::FocusChange, &f);
                    }
                }
            },
            PrimitiveEvent::KeyInput(vkc, _sc, s, _m) => {
                match vkc {
                    Some(VirtualKeyCode::Right) => {
                        if self.cursor < self.value.len() && s == properties::ButtonState::Pressed {
                            self.cursor += 1;
                        }
                    }
                    Some(VirtualKeyCode::Left) => {
                        if self.cursor > 0 && s == properties::ButtonState::Pressed {
                            self.cursor -= 1;
                        }
                    }
                    _ => ()
                }
            },
            PrimitiveEvent::HoverBegin(n_tags) => {
                let matched = n_tags.iter().find(|x|{
                    x.0 == self.ext_id
                });
                if let Some(_) =  matched {
                    self.hovering = true;
                }
            },
            PrimitiveEvent::HoverEnd(o_tags) => {
                let matched = o_tags.iter().find(|x|{
                    x.0 == self.ext_id
                });
                if let Some(_) =  matched {
                    self.hovering = false;
                }
            },
            _ => ()
        }
        return handled;
    }

    fn set_handler(&mut self, e: ElementEvent, f: EventFn) {
        self.event_handlers.insert(e, f);
    }

    fn exec_handler(&mut self, _e: ElementEvent, _d: &Any) -> bool {
        let eh = &mut self.event_handlers;
        let h = eh.get_mut(&_e);
        if let Some(h) = h{
            (h.0)(_d)
        } else {
            false
        }
    }

    fn as_any(&self) -> &Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}

impl CanDisable for TextBox {
    fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }
}

