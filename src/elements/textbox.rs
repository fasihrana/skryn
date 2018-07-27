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
              _props: Option<Arc<properties::Properties>>) {
        let size = 12.0;
        let family= String::from("Arial");
        let color = ColorF{
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        let mut _x = extent.x.clone();
        let mut _y = extent.y.clone() + (size*1.1);
        let mut glyphs = Vec::new();
        let mut ignore_ws = true;

        let fi_key = font_store.get_font_instance_key(&family,size as i32);
        let font_type = font_store.get_font_type(&family);
        let mut mappings = font_type.glyphs_for(self.value.chars());
        let mut text_iter = self.value.chars();


        loop {
            let _char = text_iter.next();
            if _char.is_none(){
                break;
            }
            let _char = _char.unwrap();
            let _glyph = mappings.next().unwrap();

            if _char == '\r' || _char == '\n' {
                _y = _y + (size*1.1);
                _x = 0.0;
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
                point: LayoutPoint::new(_x,_y)
            });

            _x=_x + ((h_metrics.advance_width+h_metrics.left_side_bearing) * size);
        }

        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(extent.x.clone(), extent.y.clone()),
            LayoutSize::new(extent.w.clone(), extent.h.clone())
        ));
        builder.push_text(&info,
                      &glyphs,
                      fi_key.clone(),
                      color.clone(),
                      Some(GlyphOptions::default()));
        }
}