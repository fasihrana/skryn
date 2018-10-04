
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

use std::sync::{Arc, Mutex};
use std::ops::DerefMut;

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

struct Internals {
    gl_window: glutin::GlWindow,
    events_loop: glutin::EventsLoop,
    font_store: Arc<Mutex<font::FontStore>>,
    api: RenderApi,
    document_id: DocumentId,
    pipeline_id: PipelineId,
    epoch: Epoch,
    renderer: webrender::Renderer,
    cursor_position: WorldPoint,
}

impl Internals{
    fn new(name: String, width: f64, height:f64) -> Internals {
        let events_loop = winit::EventsLoop::new();
        let context_builder = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0),
            });
        let window_builder = winit::WindowBuilder::new()
            .with_title(name.clone())
            .with_multitouch()
            .with_dimensions(winit::dpi::LogicalSize::new(width, height));
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

        let mut dpi = window.get_hidpi_factor();

        let opts = webrender::RendererOptions {
            device_pixel_ratio: dpi as f32,
            clear_color: Some(ColorF::new(1.0, 1.0, 1.0, 1.0)),
            //enable_scrollbars: true,
            //enable_aa:true,
            ..webrender::RendererOptions::default()
        };

        let mut framebuffer_size = {
            let size = window
                .get_inner_size()
                .unwrap()
                .to_physical(dpi);
            DeviceUintSize::new(size.width as u32, size.height as u32)
        };

        let notifier = Box::new(WindowNotifier::new(events_loop.create_proxy()));
        let (renderer, sender) = webrender::Renderer::new(gl.clone(), notifier, opts).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(framebuffer_size, 0);

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);

        let mut font_store = Arc::new(Mutex::new(font::FontStore::new(api.clone_sender().create_api(),document_id.clone())));

        font_store.lock().unwrap().get_font_instance_key(&String::from("Arial"), 12);

        Internals{
            gl_window: window,
            events_loop,
            font_store,
            api,
            document_id,
            pipeline_id,
            epoch,
            renderer,
            cursor_position: WorldPoint::new(0.0,0.0),
        }
    }

    fn events(&mut self, tags:&Vec<ItemTag>) -> Vec<PrimitiveEvent> {
        let mut events = Vec::new();

        let mut cursor_position = self.cursor_position.clone();

        let mut txn = None;

        self.events_loop.poll_events(|event|{
            match event {
                glutin::Event::WindowEvent { event: glutin::WindowEvent::CloseRequested, .. } => {
                    events.push(PrimitiveEvent::Exit);
                },
                glutin::Event::WindowEvent {event: glutin::WindowEvent::CursorEntered {device_id}, .. } => {
                    events.push(PrimitiveEvent::CursorEntered);
                },
                glutin::Event::WindowEvent {event: glutin::WindowEvent::CursorMoved {device_id, position, modifiers}, .. } => {
                    cursor_position = WorldPoint::new(position.x as f32, position.y as f32);
                    events.push(PrimitiveEvent::CursorMoved(position.into()));
                },
                glutin::Event::WindowEvent {event: glutin::WindowEvent::CursorLeft {device_id}, .. } => {
                    events.push(PrimitiveEvent::CursorLeft);
                },
                glutin::Event::WindowEvent {event:glutin::WindowEvent::Resized(size),..} => {
                    events.push(PrimitiveEvent::Resized(size));
                },
                glutin::Event::WindowEvent {event: winit::WindowEvent::HiDpiFactorChanged(factor),..} => {
                    events.push(PrimitiveEvent::DPI(factor));
                },
                winit::Event::WindowEvent {event: winit::WindowEvent::MouseInput {state, button, modifiers, ..}, ..} => {
                    let _pos : properties::Position = cursor_position.clone().into();
                    let _button = button.into();
                    let _state = state.into();
                    let _modifiers = modifiers.into();

                    if tags.len() > 0 {
                        if button == winit::MouseButton::Left && state == winit::ElementState::Released {
                            events.push(PrimitiveEvent::SetFocus(true));
                        }
                        events.push(PrimitiveEvent::Button(_pos,_button,_state,_modifiers));
                    }
                },
                winit::Event::WindowEvent {event: winit::WindowEvent::MouseWheel { delta, modifiers, ..}, ..} => {
                    if txn.is_none() {
                        txn = Some(Transaction::new());
                    }
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

                    if let Some(ref mut _txn) = txn {
                        _txn.scroll(
                            ScrollLocation::Delta(LayoutVector2D::new(dx, dy)),
                            cursor_position,
                        );
                    }

                    //println!("scrolling {} {}",dx,dy);
                },
                winit::Event::WindowEvent {event: winit::WindowEvent::ReceivedCharacter(c), ..} => {
                    if c == '\x1b' {
                        events.push(PrimitiveEvent::SetFocus(false));
                    } else {
                        events.push(PrimitiveEvent::Char(c));
                    }
                },
                _ => ()
            }
        });

        if let Some(mut _txn) = txn {
            self.api.send_transaction(self.document_id, _txn);
        }

        self.cursor_position = cursor_position;

        events
    }
}

pub struct Window {
    width: f64,
    height: f64,
    root: Box<Element>,
    name: String,
    id_generator: properties::IdGenerator,
    internals: Option<Internals>,
}

impl Window {
    pub fn new(root: Box<Element>, name: String, width: f64, height: f64) -> Window {
        let id_generator = properties::IdGenerator::new(0);

        let mut _w = Window {
            width,
            height,
            root,
            name,
            id_generator,
            internals: None,
        };

        _w.start_window();

        _w
    }

    fn start_window(&mut self){
        self.internals = Some(Internals::new(self.name.clone(),self.width,self.height));
    }

    fn get_tags(&mut self) -> Vec<ItemTag>{
        let mut tags : Vec<ItemTag> = Vec::new();
        if let Some(ref mut i) = self.internals
        {
            let results = i.api.hit_test(
                i.document_id,
                None,
                i.cursor_position,
                HitTestFlags::FIND_ALL
            );
            let mut ind = results.items.len();
            while ind > 0 {
                ind -= 1;
                tags.push(results.items[ind].tag);
            }
        }

        tags
    }



    pub fn tick(&mut self) -> bool{
        let tags = self.get_tags();
        let mut xy = WorldPoint::new(0.0,0.0);

        let mut events = vec![];

        if let Some (ref mut i) = self.internals{
            events = i.events(&tags);
            xy = i.cursor_position.clone();
        }

        //only for debug. take out later?
        if events.len() > 0 {
            println!("{:?}", events);
        }

        let mut render = false;
        let mut exit = false;

        for e in events.iter(){
            if exit {
                return true;
            }
            match e {
                PrimitiveEvent::Exit => {
                    exit = true;
                },
                PrimitiveEvent::Resized(_) => {
                    render = true;
                },
                PrimitiveEvent::CursorMoved(p) => {
                    xy = WorldPoint::new(p.x,p.y);
                },
                PrimitiveEvent::SetFocus(_) => {
                    self.root.on_primitive_event(&tags, e.clone());
                    render = true;
                },
                PrimitiveEvent::Char(_) => {
                    self.root.on_primitive_event(&tags, e.clone());
                    render = true;
                },
                PrimitiveEvent::DPI(_) => {
                    render = true;
                },
                _ => ()
            }
        }

        if !render {
            render = self.root.is_invalid();
        }

        if render {
            let mut dpi = 1.0;

            let mut txn = Transaction::new();
            let mut builder = None;
            let mut font_store = None;

            let (layout_size, framebuffer_size) = if let Some (ref mut i) = self.internals {
                unsafe {
                    i.gl_window.make_current().ok();
                }

                dpi = i.gl_window.get_hidpi_factor();
                let framebuffer_size = {
                    let size = i.gl_window
                        .get_inner_size()
                        .unwrap()
                        .to_physical(dpi);
                    DeviceUintSize::new(size.width as u32, size.height as u32)
                };
                let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(dpi as f32);

                builder = Some(DisplayListBuilder::new(i.pipeline_id, layout_size));

                font_store = Some(i.font_store.clone());

                (Some(layout_size), Some(framebuffer_size))
            } else {
                (None,None)
            };

            let mut builder = builder.unwrap();
            let font_store = font_store.unwrap();
            let mut font_store = font_store.lock().unwrap();
            let mut font_store = font_store.deref_mut();
            let framebuffer_size= framebuffer_size.unwrap();
            let layout_size = layout_size.unwrap();

            self.render_root(&mut builder,font_store,dpi as f32);

            if let Some(ref mut i) = self.internals{
                txn.set_display_list(
                    i.epoch,
                    None,
                    layout_size,
                    builder.finalize(),
                    true,
                );
                txn.set_root_pipeline(i.pipeline_id);
                txn.generate_frame();
                i.api.send_transaction(i.document_id, txn);

                i.renderer.update();
                i.renderer.render(framebuffer_size).unwrap();
                let _ = i.renderer.flush_pipeline_info();
                i.gl_window.swap_buffers().ok();
            }
        }
        exit
    }

    pub fn deinit(self) -> Box<Element> {
        /*let x = self.renderer;
        x.deinit();*/
        let x = self.root;
        x
    }

    fn render_root(&mut self, builder:&mut DisplayListBuilder, font_store:&mut font::FontStore, dpi: f32){
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
}