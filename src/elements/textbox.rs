use std::sync::Arc;
use std::any::Any;

use rusttype;
use webrender::api::*;

use elements::*;
use gui::properties;
use gui::font;

pub struct TextBox {
    value: String,
    props: properties::Properties,
    bounds: properties::Extent,
    cache:Vec<GlyphInstance>,
    focus: bool,
    event_handlers: EventHandlers,
}

impl TextBox{
    pub fn new(s: String) -> Self{
        let mut props = properties::Properties::new();
        props.default();
        TextBox{
            value:s,
            props,
            bounds: properties::Extent{
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0,
            },
            cache: Vec::new(),
            focus: false,
            event_handlers: EventHandlers::new(),
        }
    }

    pub fn set_value(&mut self, s: String){
        self.value = s;
        self.cache.clear();
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}

impl Element for TextBox{
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
              _props: Option<Arc<properties::Properties>>) {

        if self.bounds != extent {
            self.cache.clear();
        }
        let glyphs = &mut self.cache;
        let size = (self.props.get_size() as f32) * extent.dpi;
        let family = self.props.get_family();
        let mut color = self.props.get_color();
        let mut bgcolor = self.props.get_bg_color();

        if self.focus {
            color = self.props.get_focus_color();
            bgcolor = self.props.get_focus_bg_color();
        }

        let fi_key = font_store.get_font_instance_key(&family, size as i32);

        if glyphs.is_empty() {

            let mut next_x = extent.x;
            let mut next_y = extent.y + size;

            let mut ignore_ws = true;


            let font_type = font_store.get_font_type(&family);
            let v_metrics = font_type.v_metrics(rusttype::Scale { x: 1.0, y: 1.0 });
            let baseline = (size * v_metrics.ascent) - size;

            let mut mappings = font_type.glyphs_for(self.value.chars());
            let mut text_iter = self.value.chars();

            let mut max_x: f32 = 0.0;

            loop {
                let _char = text_iter.next();
                if _char.is_none() {
                    break;
                }
                let _char = _char.unwrap();
                let _glyph = mappings.next().unwrap();

                if _char == '\r' || _char == '\n' {
                    next_y = next_y + size;
                    next_x = 0.0;
                    ignore_ws = true;
                    continue;
                }
                if ignore_ws && (_char == ' ' || _char == '\t') {
                    continue;
                }
                if _glyph.id().0 == 0 {
                    continue;
                }

                ignore_ws = false;

                let _scaled = _glyph.scaled(rusttype::Scale { x: 1.0, y: 1.0 });
                let h_metrics = _scaled.h_metrics();

                glyphs.push(GlyphInstance {
                    index: _scaled.id().0,
                    point: LayoutPoint::new(next_x, next_y + baseline)
                });

                next_x = next_x + ((h_metrics.advance_width + h_metrics.left_side_bearing) * size);
                if max_x < next_x {
                    max_x = next_x;
                }
            }

            self.bounds = properties::Extent{
                x: extent.x,
                y: extent.y,
                w: max_x,
                h: next_y - extent.y,
                dpi: extent.dpi,
            };
        }

        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(extent.x, extent.y),
            LayoutSize::new(self.bounds.w, self.bounds.h)
        ));
        builder.push_rect(&info, bgcolor);

        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(extent.x, extent.y),
            LayoutSize::new(extent.w, extent.h)
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

    fn on_primitive_event(&mut self, e: PrimitiveEvent) {
        match e {
            PrimitiveEvent::Char(c) => {
                if self.focus {
                    if c == '\x08' {
                        let mut l = self.value.len();
                        if l > 0 {
                            l = l - 1;
                            while !self.value.is_char_boundary(l) && l > 0 {
                                l = l - 1;
                            }
                            self.value = self.value.split_at(l).0.to_owned();
                        }
                    } else {
                        self.value.push(c);
                    }
                    self.cache.clear();
                }
            },
            PrimitiveEvent::SetFocus(f,_p) => {
                if self.focus != f {
                    self.focus = f;
                    let handler = self.get_handler(ElementEvent::FocusChange);
                    handler(self, &f);
                }
            }
            _ => ()
        }

    }

    fn set_handler(&mut self, e: ElementEvent, f: EventFn) {
        self.event_handlers.insert(e,f);
    }

    fn get_handler(&mut self, _e: ElementEvent) -> EventFn {
        let eh = &mut self.event_handlers;
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
}

