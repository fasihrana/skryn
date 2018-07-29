use std::sync::Arc;

use rusttype;
use webrender::api::*;

use elements::element::*;
use gui::properties;
use gui::font;

pub struct TextBox {
    value: String,
    props: properties::Properties,
}

impl TextBox{
    pub fn new(s: String) -> Self{
        let mut props = properties::Properties::new();
        props.default();
        TextBox{
            value:s,
            props,
        }
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
              _props: Option<Arc<properties::Properties>>) -> properties::Extent {

        let size = (self.props.get_size() as f32) * extent.dpi;
        let family = self.props.get_family();
        let color = self.props.get_color();
        let bgcolor = self.props.get_bg_color();

        let mut next_x = extent.x;
        let mut next_y = extent.y + size;
        let mut glyphs = Vec::new();
        let mut ignore_ws = true;

        let fi_key = font_store.get_font_instance_key(&family,size as i32);
        let font_type = font_store.get_font_type(&family);
        let v_metrics = font_type.v_metrics(rusttype::Scale{ x: 1.0, y: 1.0 });
        let baseline = (size * v_metrics.ascent) - size;

        let mut mappings = font_type.glyphs_for(self.value.chars());
        let mut text_iter = self.value.chars();

        let used_extent = properties::Extent{
            x: extent.x,
            y: extent.y,
            w: 0.0,
            h: 0.0,
            dpi: extent.dpi,
        };

        let mut max_x:f32 = 0.0;

        loop {
            let _char = text_iter.next();
            if _char.is_none(){
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

            let _scaled = _glyph.scaled(rusttype::Scale{ x: 1.0, y: 1.0 });
            let h_metrics = _scaled.h_metrics();

            glyphs.push(GlyphInstance{
                index: _scaled.id().0,
                point: LayoutPoint::new(next_x,next_y + baseline)
            });

            next_x = next_x + ((h_metrics.advance_width+h_metrics.left_side_bearing) * size);
            if max_x < next_x {
                max_x = next_x;
            }
        }

        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(extent.x, extent.y),
            LayoutSize::new(max_x, next_y - extent.y)
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


        properties::Extent{
            x: extent.x,
            y: extent.y,
            w: max_x,
            h: next_y - extent.y,
            dpi: extent.dpi,
        }
    }
}