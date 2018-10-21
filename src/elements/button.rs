use std::sync::Arc;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use webrender::api::*;

use elements::element::*;
use gui::properties;
use gui::font;

pub struct Button
{
    ext_id: u64,
    value: String,
    props: properties::Properties,
    bounds: properties::Extent,
    //    cache:Vec<GlyphInstance>,
    focus: bool,
    event_handlers: EventHandlers,
    drawn: u8,
}

impl Button {
    pub fn new(s: String) -> Self {
        let mut props = properties::Properties::new();
        props.default();
        props.set(properties::Property::BgColor(ColorF::new(0.8, 0.9, 0.9, 1.0)))
            .set(properties::Property::HoverBgColor(ColorF::new(0.7, 0.8, 0.8, 1.0)));
        Button {
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
//            cache: Vec::new(),
            focus: false,
            event_handlers: EventHandlers::new(),
            drawn: 0,
        }
    }

    pub fn set_value(&mut self, s: String) {
        self.value = s;
        //self.cache.clear();
        self.drawn = 0;
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}

impl Element for Button {
    fn get_ext_id(&self) -> u64 { self.ext_id }

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
        let _id = gen.get();
        self.ext_id = _id;


        let mut glyphs = vec![];
        let size = self.props.get_size() as f32;
        let family = self.props.get_family();
        let mut color = self.props.get_color();
        let mut bgcolor = self.props.get_bg_color();
        let width = self.props.get_width();
        let height = self.props.get_height();

        if self.focus {
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

        loop {
            let _char = text_iter.next();
            if _char.is_none() {
                break;
            }
            let _char = _char.unwrap();

            if _char == '\r' || _char == '\n' {
                next_y = next_y + size;
                next_x = 0.0;
                continue;
            }

            if _char == ' ' || _char == '\t' {
                next_x += (size/3.0);
                continue;
            }

            let _glyph = mappings.get(&_char);

            if let Some((gi, gd)) = _glyph {
                glyphs.push(GlyphInstance {
                    index: gi.to_owned(),
                    point: LayoutPoint::new(next_x, next_y),
                });

                next_x = next_x + gd.advance;
            }

            if max_x < next_x {
                max_x = next_x;
            }
        }

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

        self.drawn += 1;
        if self.drawn > 2 {
            self.drawn = 2;
        }


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
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, ext_ids: &[ItemTag], e: PrimitiveEvent) -> bool {
        let mut handled = false;

        match e {
            PrimitiveEvent::Button(_p, b, s, m) => {
                self.drawn = 0;
                if ext_ids.len() == 1 && ext_ids[0].0 == self.ext_id {
                    if b == properties::Button::Left
                        && s == properties::ButtonState::Released
                        {
                            let handler = self.get_handler(ElementEvent::Clicked);
                            handled = handler(self, &m);
                        }
                }
            }
            _ => ()
        }

        return handled;
    }

    fn set_handler(&mut self, _e: ElementEvent, _f: EventFn) {
        self.event_handlers.insert(_e, _f);
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

