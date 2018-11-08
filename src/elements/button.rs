use std::sync::Arc;
use std::any::Any;

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
    //focus: bool,
    event_handlers: EventHandlers,
    drawn: u8,
    hovering: bool,
    enabled: bool,
}

impl Button {
    pub fn new(s: String) -> Self {
        let mut props = properties::Properties::new();
        props.default();
        props.set(properties::Property::BgColor(ColorF::new(0.8, 0.9, 0.9, 1.0)))
            .set(properties::Property::Color(ColorF::new(0.2, 0.2, 0.2, 1.0)))
            .set(properties::Property::HoverBgColor(ColorF::new(0.6, 0.7, 0.7, 1.0)));
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
            //focus: false,
            event_handlers: EventHandlers::new(),
            drawn: 0,
            hovering: false,
            enabled: true,
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


        let mut color = self.props.get_color();
        let mut bgcolor = self.props.get_bg_color();
        let width = self.props.get_width();
        let height = self.props.get_height();
        let size = self.props.get_size() as f32;
        let family = self.props.get_family();
        let text_align = self.props.get_text_align();

        if self.hovering && self.enabled {
            color = self.props.get_hover_color();
            bgcolor = self.props.get_hover_bg_color();
        }

        let (_, fi_key) = font_store.get_font_instance(&family, size as i32);

        let (glyphs, mut calc_w, mut calc_h) = font::FontRaster::place_line(&self.value[0..],//font::FontRaster::place_glyphs(&self.value,
                                                                     0.0,
                                                                     0.0,
                                                                     //extent.w,
                                                                     //extent.h,
                                                                     size,
                                                                     &family,
                                                                     //text_align,
                                                                     font_store);

        let mut _x = (extent.w - calc_w) / 2.0;
        let mut _y = (extent.h - calc_h) / 2.0;

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

        builder.push_stacking_context(
            &LayoutPrimitiveInfo::new(LayoutRect::new(
                LayoutPoint::new(extent.x + _x, extent.y + _y),
                LayoutSize::new(0.0, 0.0),
            )),
            None,
            TransformStyle::Flat,
            MixBlendMode::Normal,
            &vec![],
            RasterSpace::Screen,
        );

        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(0.0, 0.0),
            LayoutSize::new(extent.w, extent.h),
        ));

        builder.push_text(&info,
                          &glyphs,
                          fi_key.clone(),
                          color.clone(),
                          Some(GlyphOptions::default()));

        builder.pop_stacking_context();
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
                        && self.enabled
                        {
                            handled = self.exec_handler(ElementEvent::Clicked, &m);
                        }
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

    fn set_handler(&mut self, _e: ElementEvent, _f: EventFn) {
        self.event_handlers.insert(_e, _f);
    }

    fn exec_handler(&mut self, _e: ElementEvent, _d: &Any) -> bool {
        let h = self.event_handlers.get_mut(&_e).cloned();
        if let Some(mut h) = h{
            h.call(self, _d)
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

impl CanDisable for Button {
    fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }
}