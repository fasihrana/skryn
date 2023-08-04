use std::any::Any;
use std::sync::Arc;

use webrender::api::*;

use crate::elements::element::*;
use crate::gui::font;
use crate::gui::properties;

pub struct Button {
    ext_id: u64,
    value: Vec<char>,
    props: properties::Properties,
    bounds: properties::Extent,
    text_bounds: properties::Extent,
    event_handlers: EventHandlers,
    drawn: u8,
    hovering: bool,
    enabled: bool,
}

impl Button {
    pub fn new(s: String) -> Self {
        let mut props = properties::Properties::new();
        props.default();
        props
            .set(properties::Property::TextAlign(properties::Align::Middle))
            .set(properties::Property::BgColor(ColorF::new(
                0.8, 0.9, 0.9, 1.0,
            )))
            .set(properties::Property::Color(ColorF::new(0.2, 0.2, 0.2, 1.0)))
            .set(properties::Property::HoverBgColor(ColorF::new(
                0.6, 0.7, 0.7, 1.0,
            )));
        Button {
            ext_id: 0,
            value: s.chars().collect(),
            props,
            bounds: properties::Extent::new(),
            text_bounds: properties::Extent::new(),
            event_handlers: EventHandlers::new(),
            drawn: 0,
            hovering: false,
            enabled: true,
        }
    }

    pub fn set_value(&mut self, s: String) {
        self.value = s.chars().collect();
        //self.cache.clear();
        self.drawn = 0;
    }

    pub fn get_value(&self) -> String {
        self.value.clone().iter().collect()
    }

    fn get_width_sums(&mut self) -> (f32, f32) {
        let left = self.props.get_left();
        let right = self.props.get_right();
        let width = self.props.get_width();

        let mut stretchy: f32 = 0.0;
        let mut pixel: f32 = 0.0;

        match left {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => (),
        }

        match right {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => (),
        }

        match width {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => (),
        }

        (pixel, stretchy)
    }

    fn get_height_sums(&mut self) -> (f32, f32) {
        let top = self.props.get_top();
        let bottom = self.props.get_bottom();

        let mut stretchy: f32 = 0.0;
        let num_lines = {
            let tmp: String = self.value.iter().collect();
            tmp.lines().count() as i32
        };
        let mut pixel: f32 = (self.props.get_size() * num_lines) as f32;

        match top {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => (),
        }

        match bottom {
            properties::Unit::Stretch(_s) => stretchy += _s,
            properties::Unit::Pixel(_p) => pixel += _p,
            _ => (),
        }

        (pixel, stretchy)
    }
}

impl Element for Button {
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

        let mut color = self.props.get_color();
        let mut bgcolor = self.props.get_bg_color();
        let width = self.props.get_width();
        let height = self.props.get_height();
        let size = self.props.get_size() as f32;
        let family = self.props.get_family();
        let text_align = self.props.get_text_align();
        let top = self.props.get_top();
        let right = self.props.get_right();
        let bottom = self.props.get_bottom();
        let left = self.props.get_left();

        if self.hovering && self.enabled {
            color = self.props.get_hover_color();
            bgcolor = self.props.get_hover_bg_color();
        }

        let (wp_sum, ws_sum) = self.get_width_sums();
        let mut remaining_width = extent.w - wp_sum;
        if remaining_width < 0.0 {
            remaining_width = 0.0;
        }
        let mut w_stretchy_factor = remaining_width / ws_sum;
        if w_stretchy_factor.is_nan() || w_stretchy_factor.is_infinite() {
            w_stretchy_factor = 0.0;
        }

        let (hp_sum, hs_sum) = self.get_height_sums();
        let mut remaining_height = extent.h - hp_sum;
        if remaining_height < 0.0 {
            remaining_height = 0.0;
        }
        let mut h_stretchy_factor = remaining_height / hs_sum;
        if h_stretchy_factor.is_nan() || h_stretchy_factor.is_infinite() {
            h_stretchy_factor = 0.0;
        }

        let mut calc_x = extent.x;
        let mut calc_y = extent.y;
        let mut calc_w = extent.w;
        let mut calc_h = extent.h;

        match top {
            properties::Unit::Pixel(_p) => {
                calc_y += _p;
                calc_h -= _p;
            }
            properties::Unit::Stretch(_s) => {
                calc_y += _s * h_stretchy_factor;
                calc_h -= _s * h_stretchy_factor;
            }
            _ => (),
        }
        match bottom {
            properties::Unit::Pixel(_p) => calc_h -= _p,
            properties::Unit::Stretch(_s) => calc_h -= _s * h_stretchy_factor,
            _ => (),
        }
        match left {
            properties::Unit::Pixel(_p) => {
                calc_x += _p;
                calc_w -= _p;
            }
            properties::Unit::Stretch(_s) => {
                calc_x += _s * w_stretchy_factor;
                calc_w -= _s * w_stretchy_factor;
            }
            _ => (),
        }
        match right {
            properties::Unit::Pixel(_p) => calc_w -= _p,
            properties::Unit::Stretch(_s) => calc_w -= _s * w_stretchy_factor,
            _ => (),
        }

        let text_y = calc_y + (calc_h - self.text_bounds.h) / 2.0;
        let metrics = font_store.get_font_metrics(&family);
        let baseline = match metrics {
            Some(metrics) => {
                let tmp = metrics.ascent - metrics.descent;
                let tmp = size / tmp;
                tmp * (metrics.ascent)
            }
            None => size,
        };

        let mut paras = font::Paragraphs::from_chars(&self.value);
        paras.shape(
            calc_x,
            text_y,
            calc_w,
            calc_h,
            size,
            baseline,
            &family,
            &text_align,
        );

        self.text_bounds = paras.get_extent();

        let mut calc_w = self.text_bounds.w;
        let mut calc_h = self.text_bounds.h;

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
            LayoutPoint::new(self.text_bounds.x, self.text_bounds.y),
            LayoutSize::new(self.text_bounds.w, self.text_bounds.h),
        ));

        let glyphs = paras.glyphs();

        let (_, fi_key) = font_store.get_font_instance(&family, size as i32);
        builder.push_text(&info, &glyphs, fi_key, color, Some(GlyphOptions::default()));
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, ext_ids: &[ItemTag], e: PrimitiveEvent) -> bool {
        let mut handled = false;

        match e {
            PrimitiveEvent::Button(_p, b, s, m) => {
                self.drawn = 0;
                if ext_ids.len() == 1
                    && ext_ids[0].0 == self.ext_id
                    && b == properties::Button::Left
                    && s == properties::ButtonState::Released
                    && self.enabled
                {
                    handled = self.exec_handler(ElementEvent::Clicked, &m);
                }
            }
            PrimitiveEvent::HoverBegin(n_tags) => {
                let matched = n_tags.iter().find(|x| x.0 == self.ext_id);
                if matched.is_some() {
                    self.hovering = true;
                }
            }
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

    fn set_handler(&mut self, _e: ElementEvent, _f: EventFn) {
        self.event_handlers.insert(_e, _f);
    }

    fn exec_handler(&mut self, _e: ElementEvent, _d: &dyn Any) -> bool {
        let h = self.event_handlers.get_mut(&_e).cloned();
        if let Some(mut h) = h {
            h.call(self, _d)
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
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
