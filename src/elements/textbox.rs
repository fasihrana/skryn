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
    //cache: Vec<GlyphDimensions>,
    //char_ext: Vec<((f32,f32),(f32,f32))>,
    cache: Vec<((f32,f32),(f32,f32))>,
    focus: bool,
    event_handlers: EventHandlers,
    drawn: u8,
    editable: bool,
    enabled: bool,
    singleline: bool,
    cursor: usize,
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
        }
    }

    pub fn set_value(&mut self, s: String) {
        self.value = s;
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

    fn get(&self, prop: &properties::Property) -> Option<&properties::Property> {
        self.props.get(&prop)
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

        if self.focus && self.enabled && self.editable {
            color = self.props.get_focus_color();
            bgcolor = self.props.get_focus_bg_color();
        }

        let (f_key, fi_key) = font_store.get_font_instance(&family, size as i32);

        let mut next_x = extent.x;
        let mut next_y = extent.y + size;

        let char_set: HashSet<char> = HashSet::from_iter(self.value.chars());

        let mappings = font_store.get_glyphs(f_key, fi_key, &char_set);
        let mut text_iter = self.value.chars();

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

        let mut calc_w = max_x;
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
            LayoutSize::new(extent.w, extent.h),
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
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        ctx.set_contents(self.value.clone()).unwrap();
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
            }
            PrimitiveEvent::Button(_p, b, s, m) => {
                if ext_ids.len() > 0 && ext_ids[0].0 == self.ext_id
                    && b == properties::Button::Left
                    && s == properties::ButtonState::Released
                    {
                        self.cursor = self.get_index_at(_p.clone());
                        let handler = self.get_handler(ElementEvent::Clicked);
                        handled = handler(self, &m);
                    }
            }
            PrimitiveEvent::SetFocus(f) => {
                if self.enabled {
                    if self.focus != f {
                        self.focus = f;
                        let handler = self.get_handler(ElementEvent::FocusChange);
                        handled = handler(self, &f);
                    }
                }
            }
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
            }
            _ => ()
        }
        return handled;
    }

    fn set_handler(&mut self, e: ElementEvent, f: EventFn) {
        self.event_handlers.insert(e, f);
    }

    fn get_handler(&mut self, _e: ElementEvent) -> EventFn {
        let eh = &mut self.event_handlers;
        let h = eh.get(&_e);
        if let Some(h) = h {
            h.clone()
        } else {
            default_fn
        }
    }

    fn as_any(&self) -> &Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn is_invalid(&self) -> bool {
        if self.drawn < 2 {
            true
        } else {
            false
        }
    }
}

