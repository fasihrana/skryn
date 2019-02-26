use std::any::Any;
use std::sync::Arc;
use itertools::Itertools;

use clipboard::{ClipboardContext, ClipboardProvider};
use glutin::VirtualKeyCode;
use webrender::api::*;
//use str::strbuf::StrBuf;

use elements::element::*;
use gui::font;
use gui::properties;
use util::unicode_compose;

pub struct TextBox {
    ext_id: u64,
    value: Vec<char>,
    normalized_value: Vec<char>,
    placeholder: Vec<char>,
    props: properties::Properties,
    bounds: properties::Extent,
    cache: Vec<((f32, f32), (f32, f32))>,
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
            value: s.chars().collect(),
            normalized_value: vec![],
            placeholder: "".chars().collect(),
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
        self.value = s.chars().collect();
        self.drawn = 0;
    }

    pub fn append_value(&mut self, s: &str) {
        let mut s: Vec<char> = s.chars().collect();
        self.value.append(&mut s);
        self.drawn = 0;
    }

    pub fn get_value(&self) -> String {
        self.value.clone().iter().collect()
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

    pub fn set_is_password(&mut self, val: bool) {
        self.is_password = val;
    }

    pub fn set_singleline(&mut self, singleline: bool) {
        self.singleline = singleline;
    }

    /*pub fn get_index_at(&self, p: &properties::Position) -> usize {
        let size = self.props.get_size() as f32;
        let mut i = 0;
        let mut cursor = 0;
        while i < self.cache.len() {
            let pmin = self.cache[i as usize].0;
            let pmax = self.cache[i as usize].1;
            if p.y > (pmax.1 - size) && p.x > pmin.0 {
                //if the click is before half-x of the character, assign i
                //otherwise assign i+1
                let mid_x = ((pmax.0 - pmin.0) / 2.0) + pmin.0;
                if p.x < mid_x {
                    cursor = i;
                } else {
                    cursor = i + 1;
                }
            }
            i += 1;
        }

        cursor
    }*/

    pub fn set_placeholder(&mut self, p: String) {
        self.placeholder = p.chars().collect();
    }

    pub fn get_placeholder(&self) -> String {
        self.placeholder.clone().iter().collect()
    }
}

impl Element for TextBox {
    fn get_ext_id(&self) -> u64 {
        self.ext_id
    }

    fn set(&mut self, prop: properties::Property) {
        self.props.set(prop);
    }

    /*fn get(&self, prop: &properties::Property) -> Option<&properties::Property> {
        self.props.get(&prop)
    }*/

    fn get_properties(&self) -> properties::Properties {
        self.props.clone()
    }

    fn render(
        &mut self,
        _api: &RenderApi,
        builder: &mut DisplayListBuilder,
        extent: properties::Extent,
        font_store: &mut font::FontStore,
        _props: Option<Arc<properties::Properties>>,
        gen: &mut properties::IdGenerator,
    ) {
        /*let tmp_str_val = self.value.clone();//self.value.iter().collect();
        //let tmp_str_val = unicode_compose(&tmp_str_val).chars().collect();

        let _id = gen.get();
        self.ext_id = _id;

        let (mut cursor_x, mut cursor_y, cursor_i) = (0.0, 0.0, self.cursor);

        let size = self.props.get_size() as f32;
        let family = self.props.get_family();
        let mut color = self.props.get_color();
        let mut bgcolor = self.props.get_bg_color();
        let width = self.props.get_width();
        let height = self.props.get_height();
        let text_align = self.props.get_text_align();

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

        let (_f_key, fi_key) = font_store.get_font_instance(&family, size as i32);

        let val_str = "●".repeat(self.value.len()).chars().collect();

        let value = if !self.is_password {
            &tmp_str_val
        } else {
            &val_str
        };

        let (mut glyphs, _bounds, cache) = font::FontRaster::place_lines(
            value,
            extent.x,
            extent.y,
            extent.w,
            extent.h,
            size,
            &family,
            &text_align,
            font_store,
        );

        self.cache = cache;

        if cursor_i == 0 && self.cache.is_empty() {
            cursor_x = _bounds.x;
            cursor_y = _bounds.y + size;
        } else if cursor_i == 0 && !self.cache.is_empty() {
            cursor_x = (self.cache[0].0).0;
            cursor_y = (self.cache[0].1).1;
        } else if !self.cache.is_empty() {
            cursor_x = (self.cache[cursor_i - 1].1).0;
            cursor_y = (self.cache[cursor_i - 1].1).1;
        }

        glyphs.retain(|x| x.index != 0);

        if self.value.is_empty() && !self.placeholder.is_empty() && !self.focus {
            let placeholder = font::FontRaster::place_lines(
                &self.placeholder,
                extent.x,
                extent.y,
                extent.w,
                extent.h,
                size,
                &family,
                &text_align,
                font_store,
            );

            let info = LayoutPrimitiveInfo::new(LayoutRect::new(
                LayoutPoint::new(placeholder.1.x, placeholder.1.y),
                LayoutSize::new(placeholder.1.w, placeholder.1.h),
            ));

            if !self.hovering {
                color = self.props.get_disabled_color();
            }

            builder.push_text(
                &info,
                &placeholder.0,
                fi_key,
                color,
                Some(GlyphOptions::default()),
            );
        }

        let mut calc_w = _bounds.w;
        let mut calc_h = _bounds.h;

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
        builder.push_text(&info, &glyphs, fi_key, color, Some(GlyphOptions::default()));

        //add the cursor
        if self.focus && self.enabled && self.editable {
            let info = LayoutPrimitiveInfo::new(LayoutRect::new(
                LayoutPoint::new(cursor_x, cursor_y - size),
                LayoutSize::new(1.0, size),
            ));
            builder.push_rect(&info, color);
        }*/
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, ext_ids: &[ItemTag], e: PrimitiveEvent) -> bool {
        let mut handled = false;
        match e {
            /*PrimitiveEvent::Char(mut c) => {
                if self.focus && self.enabled && self.editable {
                    if c == '\x08' {
                        let mut l = self.cursor;
                        if l > 0 {
                            //l -= 1;
                            /*while !self.value.is_char_boundary(l) && l > 0 {
                                l -= 1;
                            }
                            self.value =
                                //format!("{}{}", &self.value[0..l], &self.value[self.cursor..]);
                            self.cursor = l;*/
                            self.value.remove(l);
                            self.cursor -=1;
                        }
                    } else if c == '\u{3}' {
                        if !self.is_password {
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            ctx.set_contents(self.value.clone()).unwrap();
                        }
                    } else if c == '\u{16}' {
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        let vstr = ctx.get_contents().unwrap();
                        self.value = format!(
                            "{}{}{}",
                            &self.value[0..self.cursor],
                            &vstr[0..],
                            &self.value[self.cursor..]
                        );
                        self.cursor += vstr.len();
                    } else {
                        if c == '\r' {
                            c = '\n';
                        }
                        let mut tmp_chars = self.value.chars().collect_vec();

                        if self.singleline && c == '\n' {
                            //do not save the new line
                        } else {
                            /*if self.cursor == 0 {
                                let mut newstr = format!("{}", c);
                                newstr.push_str(&self.value[0..]);
                                self.value = newstr;
                            } else if self.cursor < tmp_chars.len() {
                                tmp_chars.insert(self.cursor,c);
                                self.value = tmp_chars.into_iter().collect();
                            } else {
                                self.value.push(c);
                            }*/

                            if self.cursor == tmp_chars.len() {
                                self.value.push(c);
                            } else {
                                tmp_chars.insert(self.cursor,c);
                                self.value = tmp_chars.into_iter().collect();
                            }

                            self.cursor += 1;
                        }
                    }
                    handled = true;
                }
            },
            PrimitiveEvent::KeyInput(vkc, _sc, s, _m) => match vkc {
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
                _ => (),
            },*/
            PrimitiveEvent::SetFocus(f) => {
                if self.enabled && self.focus != f {
                    self.focus = f;
                    handled = self.exec_handler(ElementEvent::FocusChange, &f);
                }
            },
            PrimitiveEvent::Button(p, b, s, m) => {
                if !ext_ids.is_empty()
                    && ext_ids[0].0 == self.ext_id
                    && b == properties::Button::Left
                    && s == properties::ButtonState::Released
                    {
                        //TODO: uncomment the following
                        //self.cursor = self.get_index_at(&p);
                        handled = self.exec_handler(ElementEvent::Clicked, &m);
                    }
            },
            PrimitiveEvent::HoverBegin(n_tags) => {
                let matched = n_tags.iter().find(|x| x.0 == self.ext_id);
                if matched.is_some() {
                    self.hovering = true;
                }
            },
            PrimitiveEvent::HoverEnd(o_tags) => {
                let matched = o_tags.iter().find(|x| x.0 == self.ext_id);
                if matched.is_some() {
                    self.hovering = false;
                }
            }
            _ => (),
        }
        handled
    }

    fn set_handler(&mut self, e: ElementEvent, f: EventFn) {
        self.event_handlers.insert(e, f);
    }

    fn exec_handler(&mut self, e: ElementEvent, d: &Any) -> bool {
        let h = self.event_handlers.get_mut(&e).cloned();
        if let Some(mut h) = h {
            h.call(self, d)
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
