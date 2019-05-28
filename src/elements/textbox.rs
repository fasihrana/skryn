use std::any::Any;
use std::sync::Arc;

use clipboard::{ClipboardContext, ClipboardProvider};
use glutin::VirtualKeyCode;
use webrender::api::*;

use crate::elements::element::*;
use crate::gui::font;
use crate::gui::properties;
use crate::gui::properties::Position;

pub struct TextBox {
    ext_id: u64,
    value: Vec<char>,
    placeholder: Vec<char>,
    props: properties::Properties,
    bounds: properties::Extent,
    focus: bool,
    event_handlers: EventHandlers,
    drawn: u8,
    editable: bool,
    enabled: bool,
    singleline: bool,
    cursor: Option<(font::Char, properties::Position)>,
    cursor_index: usize,
    cursor_after: bool,
    hovering: bool,
    is_password: bool,
    cache: font::Paragraphs,
    selecting: bool,
}

impl TextBox {
    pub fn new(s: String) -> Self {
        let mut props = properties::Properties::new();
        props.default();
        props.set(properties::Property::Height(properties::Unit::Natural));
        TextBox {
            ext_id: 0,
            value: s.chars().collect(),
            placeholder: "".chars().collect(),
            props,
            bounds: properties::Extent {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0,
            },
            focus: false,
            event_handlers: EventHandlers::new(),
            drawn: 0,
            editable: true,
            enabled: true,
            singleline: false,
            cursor: None,
            cursor_index: 0,
            cursor_after: false,
            hovering: false,
            is_password: false,
            cache: font::Paragraphs::new(),
            selecting: false,
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

    pub fn get_cursor_index(&self) -> (usize,bool) {
        match self.cursor {
            None => (0,false),
            Some((ref ch, ref p)) => {
                let mut ind = ch.get_index();
                let pos = ch.get_position();
                let adv = ch.get_metric().advance;
                let mut after = false;
                if p.x > pos.x + (adv.x/2.) {
                    //ind += 1;
                    after = true;
                }
                (ind,after)
            }
        }
    }

    pub fn set_placeholder(&mut self, p: String) {
        self.placeholder = p.chars().collect();
    }

    pub fn get_placeholder(&self) -> String {
        self.placeholder.clone().iter().collect()
    }

    fn set_cursor(&mut self, p: &Position){
        let tmp = self.cache.get_char_at_pos(&p, &self.value);
        if tmp.is_some() {
            self.cursor = Some((tmp.unwrap(), p.clone()));
            let tmp = self.get_cursor_index();
            self.cursor_index = tmp.0;
            self.cursor_after = tmp.1;
            //println!("Clicked at ind[{}] {:?} ... appears after? {}", self.cursor_index, self.cursor, self.cursor_after);
        }
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
        let _id = gen.get();
        self.ext_id = _id;

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

        if self.value.is_empty() && !self.placeholder.is_empty() && !self.focus && !self.hovering {
            color = self.props.get_disabled_color();
        }

        let (_f_key, fi_key) = font_store.get_font_instance(&family, size as i32);

        let val_str = "●".repeat(self.value.len()).chars().collect();

        let value = if !self.is_password {
            &self.value
        } else {
            &val_str
        };

        let value = if value.is_empty() {
            &self.placeholder
        } else {
            &value
        };

        let metrics = font_store.get_font_metrics(&family);
        let baseline = match metrics {
            Some(metrics) => {
                let tmp = metrics.ascent - metrics.descent;
                let tmp = size / tmp;
                tmp * (metrics.ascent)
            }
            None => size,
        };

        let mut paras = font::Paragraphs::from_chars(value);
        paras.shape(
            extent.x,
            extent.y,
            extent.w,
            extent.h,
            size,
            baseline,
            &family,
            &text_align,
        );
        let _bounds = paras.get_extent();
        let glyphs = paras.glyphs();

        if !self.value.is_empty() {
            self.cache = paras;
        }

        /*let (mut cursor_x, mut cursor_y, cursor_i) = (0.0, 0.0, self.cursor);

        if cursor_i == 0 && self.cache.is_empty() {
            cursor_x = _bounds.x;
            cursor_y = _bounds.y + size;
        } else if cursor_i == 0 && !self.cache.is_empty() {
            cursor_x = (self.cache[0].0).0;
            cursor_y = (self.cache[0].1).1;
        } else if !self.cache.is_empty() {
            cursor_x = (self.cache[cursor_i - 1].1).0;
            cursor_y = (self.cache[cursor_i - 1].1).1;
        }*/

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
            let ch = self.cache.get_char_at_index(self.cursor_index);
            match ch {
                Some(ref ch) => {
                    let pos = ch.get_position();
                    let mut info = LayoutPrimitiveInfo::new(LayoutRect::new(
                        LayoutPoint::new(pos.x, pos.y - size),
                        LayoutSize::new(1.0, size),
                    ));
                    if self.cursor_after {
                        info = LayoutPrimitiveInfo::new(LayoutRect::new(
                            LayoutPoint::new(pos.x + ch.get_metric().advance.x, pos.y - size),
                            LayoutSize::new(1.0, size),
                        ));
                    }
                    builder.push_rect(&info, color);
                },
                None => (),
            }
        }
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, ext_ids: &[ItemTag], e: PrimitiveEvent) -> bool {
        let mut handled = false;
        match e {
            PrimitiveEvent::KeyInput(vkc, _sc, _s, _m) => match vkc {
                _ => (),
            },
            PrimitiveEvent::Char(mut c) => {
                if self.focus && self.enabled && self.editable {
                    if c == '\x08' { //backspace
                        let len = self.value.len();
                        //if len > 0 and after is true then should delete from index
                        if len > 0 && self.cursor_after {
                            self.value.remove(self.cursor_index);
                            if self.cursor_index == 0 {
                                self.cursor_after = false;
                            } else {
                                self.cursor_index -= 1;
                            }
                        }
                        // if len > 0 and after is false
                        else if len > 0 && !self.cursor_after {
                            if self.cursor_index > 0 {
                                self.value.remove(self.cursor_index - 1);
                                self.cursor_index -= 1;
                            }
                        }
                    } else if c == '\u{7f}' { //delete key
                        let len = self.value.len();
                        if len > self.cursor_index+1 && self.cursor_after {
                            self.value.remove(self.cursor_index+1);
                        }
                        else if len > self.cursor_index && !self.cursor_after{
                            self.value.remove(self.cursor_index);
                            let len = self.value.len();
                            if self.cursor_index > 0 && self.cursor_index == len && len > 0 {
                                self.cursor_index -=1;
                                self.cursor_after = true;
                            }
                        }

                    } else if c == '\u{3}' {

                    } else if c == '\u{16}' {

                    } else {
                        if c == '\r' {
                            c = '\n';
                        }
                        if self.cursor.is_some() {
                            if self.cursor_after {
                                self.value.insert(self.cursor_index + 1,c);
                            } else {
                                self.value.insert(self.cursor_index,c);
                            }
                            if self.value.len() == 1 {
                                self.cursor_index = 0;
                                self.cursor_after = true;
                            }
                            else {
                                self.cursor_index += 1;
                            }
                        }
                    }
                    handled = true;
                }
            },
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
                {
                    if s == properties::ButtonState::Pressed{
                        self.selecting = true;
                        self.set_cursor(&p);
                    }
                    else if s == properties::ButtonState::Released
                    {
                        self.selecting = false;
                        self.set_cursor(&p);
                        handled = self.exec_handler(ElementEvent::Clicked, &m);
                    }
                }
            },
            PrimitiveEvent::CursorMoved(p) => {

                if self.selecting && !ext_ids.is_empty() && ext_ids[0].0 == self.ext_id{

                    println!("cursor moved");
                    self.set_cursor(&p);

                    /*let tmp = self.cache.get_char_at_pos(&p, &self.value);
                    if tmp.is_some() {
                        self.cursor = Some((tmp.unwrap(), p.clone()));
                        let tmp = self.get_cursor_index();
                        self.cursor_index = tmp.0;
                        self.cursor_after = tmp.1;
                        println!("Clicked at ind[{}] {:?} ... appears after? {}", self.cursor_index, self.cursor, self.cursor_after);
                    }*/
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
