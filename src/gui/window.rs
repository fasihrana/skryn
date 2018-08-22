
use gleam::gl;
use glutin;
use glutin::GlContext;
use winit;
use webrender;
use webrender::api::*;
use euclid;

use util::*;
use elements::{Element, PrimitiveEvent};
use gui::font;
use gui::properties;

impl Into<properties::Position> for winit::dpi::LogicalPosition {
    fn into(self) -> properties::Position {
        properties::Position{
            x:self.x as f32,
            y:self.y as f32,
        }
    }
}

impl Into<properties::Position> for WorldPoint {
    fn into(self) -> properties::Position {
        match self {
            WorldPoint{x,y,_unit} => properties::Position{
                x: x,
                y: y,
            }
        }
    }
}

impl Into<properties::Modifiers> for winit::ModifiersState {
    fn into(self) -> properties::Modifiers {
        properties::Modifiers{
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            logo: self.logo,
        }
    }
}

impl Into<properties::Button> for winit::MouseButton{
    fn into(self) -> properties::Button{
        match self {
            winit::MouseButton::Left => {
                properties::Button::Left
            },
            winit::MouseButton::Right => {
                properties::Button::Right
            },
            winit::MouseButton::Middle => {
                properties::Button::Middle
            },
            winit::MouseButton::Other(_)=> {
                properties::Button::Other
            },
        }
    }
}

impl Into<properties::ButtonState> for winit::ElementState{
    fn into (self) -> properties::ButtonState{
        match self {
            winit::ElementState::Pressed => {
                properties::ButtonState::Pressed
            },
            winit::ElementState::Released => {
                properties::ButtonState::Released
            },
        }
    }
}

struct WindowNotifier {
    events_proxy: winit::EventsLoopProxy,
}

impl WindowNotifier {
    fn new(events_proxy: winit::EventsLoopProxy) -> WindowNotifier {
        WindowNotifier { events_proxy }
    }
}

impl RenderNotifier for WindowNotifier {
    fn clone(&self) -> Box<RenderNotifier> {
        Box::new(WindowNotifier {
            events_proxy: self.events_proxy.clone(),
        })
    }

    fn wake_up(&self) {
        #[cfg(not(target_os = "android"))]
        let _ = self.events_proxy.wakeup();
    }

    fn new_frame_ready(&self,
                       _doc_id: DocumentId,
                       _scrolled: bool,
                       _composite_needed: bool,
                       _render_time: Option<u64>) {
        self.wake_up();
    }
}



pub struct Window {
    width: f64,
    height: f64,
    root: Box<Element>,
    name: String,
    cursor_position: WorldPoint,
    id_generator: properties::IdGenerator,
}

impl Window {
    pub fn new(root: Box<Element>, name: String, width: f64, height: f64) -> Window {
        let id_generator = properties::IdGenerator::new(0);
        Window {
            width,
            height,
            root,
            name,
            cursor_position: WorldPoint::new(0.0,0.0),
            id_generator,
        }
    }

    /*#[allow(unused)]
    fn events(&mut self, ext_ids:Vec<ItemTag>) -> Vec<PrimitiveEvent> {
        let mut events = Vec::new();

        /*self.events_loop.poll_events(|event|{
            match event {
                winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. /*window_id*/ } => {
                    events.push(PrimitiveEvent::Exit);
                },
                winit::Event::WindowEvent {event: winit::WindowEvent::CursorEntered {device_id}, .. } => {
                    events.push(PrimitiveEvent::CursorEntered);
                },
                winit::Event::WindowEvent {event: winit::WindowEvent::CursorMoved {device_id, position, modifiers}, .. } => {
                    events.push(PrimitiveEvent::CursorMoved(position.into()));
                },
                winit::Event::WindowEvent {event: winit::WindowEvent::CursorLeft {device_id}, .. } => {
                    events.push(PrimitiveEvent::CursorLeft);
                },
                winit::Event::WindowEvent {event: winit::WindowEvent::MouseInput {device_id, state, button, modifiers}, ..} => {
                    let _tmp = mouse_position_cache.clone();
                    if let Some(mp) = _tmp {
                        events.push(PrimitiveEvent::SetFocus(true,Some(mp.clone())));
                        events.push(PrimitiveEvent::Button(mp,button.into(), state.into(), modifiers.into()));
                    }
                },
                winit::Event::WindowEvent {event: winit::WindowEvent::MouseWheel {device_id, delta, phase, modifiers},..} => {
                    const LINE_HEIGHT:f32 = 40.0;

                    let (dx,dy) = match delta {
                        winit::MouseScrollDelta::LineDelta(dx,dy) => (dx, dy*LINE_HEIGHT),
                        winit::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                    };

                    events.push(PrimitiveEvent::Scroll(dx,dy));
                },
                /*winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput {device_id,input}, ..} => {
                    println!("{:?}", input);
                },*/
                winit::Event::WindowEvent {event: winit::WindowEvent::ReceivedCharacter(c), ..} => {
                    if c == '\x1b' {
                        events.push(PrimitiveEvent::SetFocus(false,None));
                    } else {
                        events.push(PrimitiveEvent::Char(c));
                    }
                },
                _ => ()
            }
        });*/
        events
    }*/

    fn render(&mut self, builder:&mut DisplayListBuilder, font_store: &mut font::FontStore, dpi: f32){
        let mut gen = self.id_generator.clone();
        gen.zero();

        let info = LayoutPrimitiveInfo::new(
            (0.0, 0.0).by(self.width as f32, self.height as f32)
        );
        builder.push_stacking_context(
            &info,
            None,
            TransformStyle::Flat,
            MixBlendMode::Normal,
            Vec::new(),
            GlyphRasterSpace::Screen,
        );

        self.root.render(builder, properties::Extent {
            x: 0.0,
            y: 0.0,
            w: self.width as f32,
            h: self.height as f32,
            dpi,
        }, font_store, None, &mut gen);

        builder.pop_stacking_context();
    }

    pub fn start(&mut self) {
        let mut events_loop = winit::EventsLoop::new();
        let context_builder = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0),
            });
        let window_builder = winit::WindowBuilder::new()
            .with_title(self.name.clone())
            .with_multitouch()
            .with_dimensions(winit::dpi::LogicalSize::new(self.width, self.height));
        let window = glutin::GlWindow::new(window_builder, context_builder, &events_loop)
            .unwrap();

        unsafe {
            window.make_current().ok();
        }

        let gl = match window.get_api() {
            glutin::Api::OpenGl => unsafe {
                gl::GlFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::OpenGlEs => unsafe {
                gl::GlesFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::WebGl => unimplemented!(),
        };

        let mut device_pixel_ratio = window.get_hidpi_factor() as f32;

        let opts = webrender::RendererOptions {
            device_pixel_ratio,
            clear_color: Some(ColorF::new(1.0, 1.0, 1.0, 1.0)),
            ..webrender::RendererOptions::default()
        };

        let mut framebuffer_size = {
            let size = window
                .get_inner_size()
                .unwrap()
                .to_physical(device_pixel_ratio as f64);
            DeviceUintSize::new(size.width as u32, size.height as u32)
        };

        let notifier = Box::new(WindowNotifier::new(events_loop.create_proxy()));
        let (mut renderer, sender) = webrender::Renderer::new(gl.clone(), notifier, opts).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(framebuffer_size, 0);

        let mut font_store = font::FontStore::new(api.clone_sender().create_api(),document_id.clone());

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);
        let mut layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);
        let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
        let mut txn = Transaction::new();


        self.render(&mut builder,
                    &mut font_store,
                    device_pixel_ratio);

        txn.set_display_list(
            epoch,
            None,
            layout_size,
            builder.finalize(),
            true,
        );
        txn.set_root_pipeline(pipeline_id);
        txn.generate_frame();
        api.send_transaction(document_id, txn);

        events_loop.run_forever(|e|{
            let mut txn = Transaction::new();
            let mut new_render = false;

            match e {
                winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. } => {
                    return winit::ControlFlow::Break;
                },
                winit::Event::WindowEvent {event,..}=> match event {
                    winit::WindowEvent::Resized(..) => {
                        framebuffer_size = {
                            let size = window
                                .get_inner_size()
                                .unwrap()
                                .to_physical(device_pixel_ratio as f64);
                            self.width = size.width;
                            self.height = size.height;
                            DeviceUintSize::new(size.width as u32, size.height as u32)
                        };
                        layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);
                        txn.set_window_parameters(
                            framebuffer_size,
                            DeviceUintRect::new(DeviceUintPoint::zero(), framebuffer_size),
                            1.0
                        );
                        new_render = true;
                    },
                    winit::WindowEvent::HiDpiFactorChanged(factor) => {
                        device_pixel_ratio = factor as f32;
                        new_render = true;
                    },
                    winit::WindowEvent::CursorMoved { position: winit::dpi::LogicalPosition { x, y }, .. } => {
                        self.cursor_position = WorldPoint::new(x as f32, y as f32);
                    },
                    winit::WindowEvent::MouseWheel { delta, modifiers,.. } => {
                        let mut _txn = Transaction::new();
                        const LINE_HEIGHT: f32 = 38.0;
                        let (dx, dy) = match modifiers.alt {
                            true => {
                                match delta {
                                    winit::MouseScrollDelta::LineDelta(_, dy) => (dy * LINE_HEIGHT, 0.0),
                                    winit::MouseScrollDelta::PixelDelta(pos) => (pos.y as f32, 0.0),
                                }
                            },
                            _ => {
                                match delta {
                                    winit::MouseScrollDelta::LineDelta(_, dy) => (0.0, dy * LINE_HEIGHT),
                                    winit::MouseScrollDelta::PixelDelta(pos) => (0.0, pos.y as f32),
                                }
                            }
                        };

                        _txn.scroll(
                            ScrollLocation::Delta(LayoutVector2D::new(dx, dy)),
                            self.cursor_position,
                        );
                        api.send_transaction(document_id,_txn);
                        //println!("scrolling {} {}",dx,dy);
                    },
                    winit::WindowEvent::MouseInput { state, button, modifiers, .. } => {
                        let mut tags : Vec<ItemTag> = Vec::new();
                        let results = api.hit_test(
                            document_id,
                            None,
                            self.cursor_position,
                            HitTestFlags::FIND_ALL
                        );

                        for item in &results.items {
                            tags.push(item.tag);
                        }

                        tags.reverse();

                        let _pos : properties::Position = self.cursor_position.clone().into();/*match self.cursor_position {
                            WorldPoint{x,y,_unit} => properties::Position{x:x,y:y},
                        };*/
                        let _button = button.into();
                        let _state = state.into();
                        let _modifiers = modifiers.into();

                        if tags.len() > 0 {
                            new_render = self.root.on_primitive_event(tags.clone(),
                                                                      PrimitiveEvent::Button(_pos,
                                                                                             _button,
                                                                                             _state,
                                                                                             _modifiers));
                        }

                    },
                    _ => {
                        new_render = self.root.on_event(event, &api, document_id);
                    }
                },
                _ => (),
            }

            if new_render {

                //do two passes of render for all the bounds to be properly calculated.
                let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
                self.render(&mut builder,
                            &mut font_store,
                            device_pixel_ratio);

                txn.set_display_list(
                    epoch,
                    None,
                    layout_size,
                    builder.finalize(),
                    true,
                );
                //txn.generate_frame();
                api.send_transaction(document_id, txn);

                txn = Transaction::new();
                builder = DisplayListBuilder::new(pipeline_id, layout_size);
                self.render(&mut builder,
                            &mut font_store,
                            device_pixel_ratio);

                txn.set_display_list(
                    epoch,
                    None,
                    layout_size,
                    builder.finalize(),
                    true,
                );
                txn.generate_frame();
            }

            api.send_transaction(document_id, txn);

            renderer.update();
            renderer.render(framebuffer_size).unwrap();
            let _ = renderer.flush_pipeline_info();
            window.swap_buffers().ok();

            return winit::ControlFlow::Continue;
        });

        renderer.deinit();
    }
}