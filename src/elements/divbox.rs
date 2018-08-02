use std::sync::Arc;
use std::any::Any;

use webrender::api::*;

use euclid::SideOffsets2D;

use util::*;
use elements::element::*;
use gui::properties;
use gui::font;
use winit;

pub struct DivBox{
    children: Vec<Box<Element>>,
    props: properties::Properties,
    bounds: properties::Extent,
    handlers: EventHandlers,
    id_generator: Option<properties::IdGenerator>,
}

impl DivBox{
    pub fn new() -> Self{
        let mut props = properties::Properties::new();
        props.default();
        DivBox{
            children:Vec::new(),
            props,
            bounds: properties::Extent{
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                dpi: 0.0,
            },
            handlers: EventHandlers::new(),
            id_generator: None,
        }
    }
}

impl Element for DivBox {
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
              props: Option<Arc<properties::Properties>>,
              gen: &mut properties::IdGenerator) {

        let bgcolor = self.props.get_bg_color();

        let info = LayoutPrimitiveInfo::new(
            LayoutRect::new(LayoutPoint::zero(), builder.content_size())
        );

        builder.push_stacking_context(
            &info,
            None,
            TransformStyle::Flat,
            MixBlendMode::Normal,
            Vec::new(),
            GlyphRasterSpace::Screen,
        );

        if true {
            // set the scrolling clip
            let clip_id = builder.define_scroll_frame(
                None,
                (0, 0).by(2000,2000),
                (extent.x, extent.y).by(extent.w, extent.h),
                vec![],
                None,
                ScrollSensitivity::ScriptAndInputEvents,
            );
            builder.push_clip_id(clip_id);

            // now put some content into it.
            // start with a white background
            let mut info = LayoutPrimitiveInfo::new((extent.x, extent.y).by(extent.w, extent.h));
            //let id = {self.id_generator};
            info.tag = Some((gen.get(), 0));
            builder.push_rect(&info, bgcolor);

            // let's make a 50x50 blue square as a visual reference
            let mut info = LayoutPrimitiveInfo::new((0, 0).to(50, 50));
            info.tag = Some((gen.get(), 0));
            builder.push_rect(&info, ColorF::new(0.0, 0.0, 1.0, 1.0));



            builder.pop_clip_id(); // clip_id
        }

        builder.pop_stacking_context();

        self.bounds = properties::Extent{
            x: extent.x,
            y: extent.y,
            w: extent.w,
            h: extent.y,
            dpi: extent.dpi,
        };
    }

    fn get_bounds(&self) -> properties::Extent {
        self.bounds.clone()
    }

    fn on_primitive_event(&mut self, e: PrimitiveEvent) -> bool {
        let mut handled = false;
        for elm in self.children.iter_mut() {
            match e.clone() {
                PrimitiveEvent::Button(p,_b,_s,_m) => {
                    if !handled {
                        let _b = elm.get_bounds();
                        if p.x >= _b.x && p.x <= (_b.w + _b.x)
                            && p.y >= _b.y && p.y <= (_b.h + _b.y) {
                            handled = elm.on_primitive_event(e.clone());
                        }
                    }
                },
                PrimitiveEvent::Char(_c) => {
                    handled = elm.on_primitive_event(e.clone());
                },
                PrimitiveEvent::SetFocus(_f,p) => {
                    if let Some(p) = p {
                        let _b = elm.get_bounds();
                        if p.x >= _b.x && p.x <= (_b.w + _b.x)
                            && p.y >= _b.y && p.y <= (_b.h + _b.y) {
                            handled = elm.on_primitive_event(e.clone());
                        } else {
                            handled = elm.on_primitive_event(PrimitiveEvent::SetFocus(false, None));
                        }
                    } else {
                        handled = elm.on_primitive_event(PrimitiveEvent::SetFocus(false, None));
                    }
                },
                _ => ()
            }
        }
        if !handled {
            match e.clone() {
                PrimitiveEvent::Button(_p,_b,_s,_m) => {
                    if _s == properties::ButtonState::Released {
                        let handler = self.get_handler(ElementEvent::Clicked);
                        handled = handler(self, &_m);
                    }
                },
                _ => ()
            }
        }
        return handled;
    }

    fn set_handler(&mut self, _e: ElementEvent, _f: EventFn) {
        self.handlers.insert(_e, _f);
    }

    fn get_handler(&mut self, _e: ElementEvent) -> EventFn {
        let eh = &mut self.handlers;
        let h = eh.get(&_e);
        if let Some(h) = h{
            h.clone()
        } else {
            default_fn
        }
    }

    fn as_any(&self) -> &Any {
        self
    }

    /*fn on_event(&mut self, event: winit::WindowEvent, api: &RenderApi, document_id: DocumentId) -> bool {
        let mut txn = Transaction::new();
        match event {
            winit::WindowEvent::KeyboardInput {
                input: winit::KeyboardInput {
                    state: winit::ElementState::Pressed,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            } => {
                let offset = match key {
                    winit::VirtualKeyCode::Down => (0.0, -10.0),
                    winit::VirtualKeyCode::Up => (0.0, 10.0),
                    winit::VirtualKeyCode::Right => (-10.0, 0.0),
                    winit::VirtualKeyCode::Left => (10.0, 0.0),
                    _ => return false,
                };

                txn.scroll(
                    ScrollLocation::Delta(LayoutVector2D::new(offset.0, offset.1)),
                    self.cursor_position,
                );
            }
            winit::WindowEvent::CursorMoved { position: winit::dpi::LogicalPosition { x, y }, .. } => {
                self.cursor_position = WorldPoint::new(x as f32, y as f32);
            }
            winit::WindowEvent::MouseWheel { delta, .. } => {
                const LINE_HEIGHT: f32 = 38.0;
                let (dx, dy) = match delta {
                    winit::MouseScrollDelta::LineDelta(dx, dy) => (dx, dy * LINE_HEIGHT),
                    winit::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };

                txn.scroll(
                    ScrollLocation::Delta(LayoutVector2D::new(dx, dy)),
                    self.cursor_position,
                );
            }
            winit::WindowEvent::MouseInput { .. } => {
                let results = api.hit_test(
                    document_id,
                    None,
                    self.cursor_position,
                    HitTestFlags::FIND_ALL
                );

                println!("Hit test results:");
                for item in &results.items {
                    println!("  • {:?}", item);
                }
                println!("");
            }
            _ => (),
        }

        api.send_transaction(document_id, txn);

        false
    }*/
}

impl HasChildren for DivBox {
    #[allow(unused)]
    fn get_child(&self, i:u32) -> Option<&Element> {None}
    #[allow(unused)]
    fn get_child_mut(&mut self, i:u32) -> Option<&mut Element> {None}
    fn append(&mut self, e:Box<Element>) {
        //e.set_id_generator()
        self.children.push(e);
    }

}