
use gleam::gl;
use glutin;
use glutin::GlContext;
use winit;
use webrender;
use webrender::api::*;
use euclid;

use elements::element;
use elements::element::PrimitiveEvent;
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
    window: glutin::GlWindow,
    events_loop: winit::EventsLoop,
    renderer: webrender::Renderer,
    pipeline_id: PipelineId,
    document_id: DocumentId,
    epoch: Epoch,
    api: RenderApi,
    font_store: font::FontStore,
    root: Option<Box<element::Element>>,
    mouse_position_cache: Option<properties::Position>,
}

impl Window {
    pub fn new(name: String, w: f64, h: f64) -> Window {
        let events_loop = winit::EventsLoop::new();
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
            device_pixel_ratio,
            ..webrender::RendererOptions::default()
        };

        let framebuffer_size = {
            let winit::dpi::PhysicalSize{width, height} = window.get_current_monitor().get_dimensions();
            DeviceUintSize::new((width*_dpi) as u32, (height*_dpi) as u32)
        };

        let notifier = Box::new(WindowNotifier::new(events_loop.create_proxy()));
        let (renderer, sender) = webrender::Renderer::new(gl.clone(), notifier, opts).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(framebuffer_size, 0);

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);

        let font_store = font::FontStore::new(api.clone_sender().create_api(),document_id.clone());

        Window {
            window,
            events_loop,
            document_id,
            pipeline_id,
            api,
            epoch,
            renderer,
            font_store,
            root: None,
            mouse_position_cache: None,
        }
    }

    pub fn deinit(self){
        let x = self.renderer;
        x.deinit();
    }

    fn render (&mut self){
        if let Some(ref mut r) = self.root {
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

            txn.set_root_pipeline(self.pipeline_id);
            txn.generate_frame();

            self.api.send_transaction(self.document_id, txn);

            renderer.update();
            renderer.render(frame_size).unwrap();
            self.window.swap_buffers().ok();
        }
    }

    fn events(&mut self, mouse_position_cache: Option<properties::Position>) -> Vec<PrimitiveEvent> {
        let mut events = Vec::new();

        self.events_loop.poll_events(|event|{
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
                /*winit::Event::WindowEvent {event: winit::WindowEvent::KeyboardInput {device_id,input}, ..} => {
                    println!("{:?}", input);
                },*/
                winit::Event::WindowEvent {event: winit::WindowEvent::ReceivedCharacter(c), ..} => {
                    events.push(PrimitiveEvent::Char(c));
                },
                _ => ()
            }
        });
        events
    }

    pub fn tick(&mut self) -> bool {
        let mp_cache = self.mouse_position_cache.clone();
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
                    _ => {_r.on_event(e.clone())}
                }
            }
        }

        self.render();

        return false;
    }

    pub fn set_root(&mut self, r: Box<element::Element> ){
        self.root = Some(r);
    }
}