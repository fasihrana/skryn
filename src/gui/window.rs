
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

use std::path::PathBuf;

impl Into<properties::Position> for winit::dpi::LogicalPosition {
    fn into(self) -> properties::Position {
        properties::Position{
            x:self.x as f32,
            y:self.y as f32,
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
    /*window: glutin::GlWindow,
    events_loop: winit::EventsLoop,
    renderer: webrender::Renderer,
    pipeline_id: PipelineId,
    document_id: DocumentId,
    epoch: Epoch,
    api: RenderApi,
    font_store: font::FontStore,
    root: Option<Box<Element>>,
    mouse_position_cache: Option<properties::Position>,
    dx: f32,
    dy: f32,*/
}

impl Window {
    pub fn new(name: String, w: f64, h: f64) -> Window {
        /*let events_loop = winit::EventsLoop::new();
        let _ctxbldr = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0),
            });
        let _winbldr = winit::WindowBuilder::new()
            .with_title(name)
            .with_dimensions(winit::dpi::LogicalSize::new(w, h));
        let window = glutin::GlWindow::new(_winbldr, _ctxbldr, &events_loop).unwrap();

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

        let _dpi = window.get_hidpi_factor();
        let device_pixel_ratio = _dpi as f32;

        let opts = webrender::RendererOptions {
            precache_shaders: false,
            device_pixel_ratio,
            ..webrender::RendererOptions::default()
        };

        let framebuffer_size = {
            let winit::dpi::PhysicalSize{width, height} = window
                .get_inner_size()
                .unwrap()
                .to_physical(_dpi);//get_current_monitor().get_dimensions();
            DeviceUintSize::new(width as u32, height as u32)
        };

        let notifier = Box::new(WindowNotifier::new(events_loop.create_proxy()));
        let (renderer, sender) = webrender::Renderer::new(gl.clone(), notifier, opts).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(framebuffer_size, 0);

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);

        let font_store = font::FontStore::new(api.clone_sender().create_api(),document_id.clone());

        let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);
        let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
        let mut txn = Transaction::new();
        txn.set_display_list(
            epoch,
            None,
            layout_size,
            builder.finalize(),
            true,
        );
        txn.set_root_pipeline(pipeline_id);
        txn.generate_frame();
        api.send_transaction(document_id, txn);*/

        Window {
            /*window,
            events_loop,
            document_id,
            pipeline_id,
            api,
            epoch,
            renderer,
            font_store,
            root: None,
            mouse_position_cache: None,
            dx: 0.0,
            dy: 0.0,*/
        }
    }

    pub fn deinit(self){
        //let x = self.renderer;
        //x.deinit();
    }

    fn render (&mut self){
        /*if let Some(ref mut r) = self.root {
            unsafe {self.window.make_current().unwrap();}

            let renderer = &mut self.renderer;

            let _dpi = self.window.get_hidpi_factor();

            let frame_size = {
                let winit::dpi::LogicalSize{width, height} = self.window.get_inner_size().unwrap();
                DeviceUintSize::new((width * _dpi) as u32,(height * _dpi) as u32)
            };

            let device_pixel_ratio = _dpi as f32;
            let layout_size = frame_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);

            let mut txn = Transaction::new();
            let mut builder = DisplayListBuilder::new(self.pipeline_id, layout_size);

            let (_width,_height) = (layout_size.to_f32().width_typed().get(),
                                    layout_size.to_f32().height_typed().get());

            let _used_extent = r.render(&mut builder,properties::Extent{
                x: 0.0,
                y: 0.0,
                w: _width.clone(),
                h: _height.clone(),
                dpi: device_pixel_ratio,
            },&mut self.font_store, None);

            txn.set_display_list(
                self.epoch,
                None,
                layout_size,
                builder.finalize(),
                true,
            );

            txn.generate_frame();

            self.api.send_transaction(self.document_id, txn);

            renderer.update();
            renderer.render(frame_size).unwrap();
            let _ = renderer.flush_pipeline_info();
            self.window.swap_buffers().ok();
        }*/
    }

    #[allow(unused)]
    fn events(&mut self, mouse_position_cache: Option<properties::Position>) -> Vec<PrimitiveEvent> {
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
    }

    pub fn tick(&mut self, mut root: impl Element) -> bool {
        /*let mp_cache = self.mouse_position_cache.clone();
        let events = self.events(mp_cache.clone());

        if let Some(ref mut _r) = self.root {

            let _bounds = _r.get_bounds();

            for e in events.iter() {
                match e {
                    PrimitiveEvent::Exit => {
                        return true;
                    },
                    PrimitiveEvent::CursorMoved(p) => {
                        self.mouse_position_cache = Some(p.clone());
                    },
                    PrimitiveEvent::Scroll(dx, dy)=> {
                        self.dx += dx;
                        self.dy += dy;
                        println!("scrolling {},{}",dx,dy);
                        if let Some(ref mp) = self.mouse_position_cache {
                            let mut txn = Transaction::new();
                            let p = WorldPoint::new(mp.x, mp.y);
                            txn.scroll(ScrollLocation::Delta(LayoutVector2D::new(*dx, *dy)), p);
                            self.api.send_transaction(self.document_id, txn);
                        }
                    },
                    _ => {
                        _r.on_primitive_event(e.clone());
                    }
                }
            }
        }

        self.render();*/


        let options = None;
        let res_path: Option<PathBuf> = None;
        let mut events_loop = winit::EventsLoop::new();
        let context_builder = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0),
            });
        let window_builder = winit::WindowBuilder::new()
            .with_title(String::from("hello?"))
            .with_multitouch()
            .with_dimensions(winit::dpi::LogicalSize::new(800 as f64, 800 as f64));
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

        println!("OpenGL version {}", gl.get_string(gl::VERSION));
        println!("Shader resource path: {:?}", res_path);
        let device_pixel_ratio = window.get_hidpi_factor() as f32;
        println!("Device pixel ratio: {}", device_pixel_ratio);

        println!("Loading shaders...");
        let opts = webrender::RendererOptions {
            device_pixel_ratio,
            clear_color: Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
            ..options.unwrap_or(webrender::RendererOptions::default())
        };

        let framebuffer_size = {
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

        let (external, output) = (None,None);//example.get_image_handlers(&*gl);

        if let Some(output_image_handler) = output {
            renderer.set_output_image_handler(output_image_handler);
        }

        if let Some(external_image_handler) = external {
            renderer.set_external_image_handler(external_image_handler);
        }

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);
        let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(device_pixel_ratio);
        let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
        let mut txn = Transaction::new();


            root.render(&mut builder, properties::Extent {
                x: 0.0,
                y: 0.0,
                w: framebuffer_size.width as f32,
                h: framebuffer_size.height as f32,
                dpi: device_pixel_ratio,
            }, &mut font_store, None);

        /*example.render(
            &api,
            &mut builder,
            &mut txn,
            framebuffer_size,
            pipeline_id,
            document_id,
        );*/
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

        println!("Entering event loop");
        events_loop.run_forever(|global_event| {
            let mut txn = Transaction::new();
            let mut custom_event = true;

            match global_event {
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::CloseRequested,
                    ..
                } => return winit::ControlFlow::Break,
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::KeyboardInput {
                        input: winit::KeyboardInput {
                            state: winit::ElementState::Pressed,
                            virtual_keycode: Some(key),
                            ..
                        },
                        ..
                    },
                    ..
                } => match key {
                    winit::VirtualKeyCode::Escape => return winit::ControlFlow::Break,
                    winit::VirtualKeyCode::P => renderer.toggle_debug_flags(webrender::DebugFlags::PROFILER_DBG),
                    winit::VirtualKeyCode::O => renderer.toggle_debug_flags(webrender::DebugFlags::RENDER_TARGET_DBG),
                    winit::VirtualKeyCode::I => renderer.toggle_debug_flags(webrender::DebugFlags::TEXTURE_CACHE_DBG),
                    winit::VirtualKeyCode::S => renderer.toggle_debug_flags(webrender::DebugFlags::COMPACT_PROFILER),
                    winit::VirtualKeyCode::Q => renderer.toggle_debug_flags(
                        webrender::DebugFlags::GPU_TIME_QUERIES | webrender::DebugFlags::GPU_SAMPLE_QUERIES
                    ),
                    winit::VirtualKeyCode::Key1 => txn.set_window_parameters(
                        framebuffer_size,
                        DeviceUintRect::new(DeviceUintPoint::zero(), framebuffer_size),
                        1.0
                    ),
                    winit::VirtualKeyCode::Key2 => txn.set_window_parameters(
                        framebuffer_size,
                        DeviceUintRect::new(DeviceUintPoint::zero(), framebuffer_size),
                        2.0
                    ),
                    winit::VirtualKeyCode::M => api.notify_memory_pressure(),
                    #[cfg(feature = "capture")]
                    winit::VirtualKeyCode::C => {
                        let path: PathBuf = "../captures/example".into();
                        //TODO: switch between SCENE/FRAME capture types
                        // based on "shift" modifier, when `glutin` is updated.
                        let bits = CaptureBits::all();
                        api.save_capture(path, bits);
                    },
                    _ => {
                        let win_event = match global_event {
                            winit::Event::WindowEvent { event, .. } => event,
                            _ => unreachable!()
                        };
                        custom_event = root.on_event(
                            win_event,
                            &api,
                            document_id,
                        )
                    },
                },
                winit::Event::WindowEvent { event, .. } => custom_event = root.on_event(
                    event,
                    &api,
                    document_id,
                ),
                _ => return winit::ControlFlow::Continue,
            };

            if custom_event {
                let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);


                root.render(&mut builder, properties::Extent {
                        x: 0.0,
                        y: 0.0,
                        w: framebuffer_size.width as f32,
                        h: framebuffer_size.height as f32,
                        dpi: device_pixel_ratio,
                    }, &mut font_store, None);
                /*example.render(
                    &api,
                    &mut builder,
                    &mut txn,
                    framebuffer_size,
                    pipeline_id,
                    document_id,
                );*/
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
            //example.draw_custom(&*gl);
            window.swap_buffers().ok();

            winit::ControlFlow::Continue
        });

        renderer.deinit();


        return false;
    }

    pub fn set_root(&mut self, r: Box<Element> ){
        //self.root = Some(r);
    }
}