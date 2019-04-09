use euclid;
use gleam::gl;
use glutin;
use glutin::ContextWrapper;
use webrender;
use webrender::api::*;

use crate::elements::{Element, PrimitiveEvent};
use crate::gui::font;
use crate::gui::properties;
use crate::util::*;

use std::mem;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use std::fmt;

impl Into<properties::Position> for glutin::dpi::LogicalPosition {
    fn into(self) -> properties::Position {
        properties::Position {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl Into<properties::Position> for glutin::dpi::PhysicalPosition {
    fn into(self) -> properties::Position {
        properties::Position {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl Into<properties::Position> for WorldPoint {
    fn into(self) -> properties::Position {
        match self {
            WorldPoint { x, y, _unit } => properties::Position { x, y },
        }
    }
}

impl Into<properties::Modifiers> for glutin::ModifiersState {
    fn into(self) -> properties::Modifiers {
        properties::Modifiers {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            logo: self.logo,
        }
    }
}

impl Into<properties::Button> for glutin::MouseButton {
    fn into(self) -> properties::Button {
        match self {
            glutin::MouseButton::Left => properties::Button::Left,
            glutin::MouseButton::Right => properties::Button::Right,
            glutin::MouseButton::Middle => properties::Button::Middle,
            glutin::MouseButton::Other(_) => properties::Button::Other,
        }
    }
}

impl Into<properties::ButtonState> for glutin::ElementState {
    fn into(self) -> properties::ButtonState {
        match self {
            glutin::ElementState::Pressed => properties::ButtonState::Pressed,
            glutin::ElementState::Released => properties::ButtonState::Released,
        }
    }
}

struct WindowNotifier {
    events_proxy: glutin::EventsLoopProxy,
}

impl WindowNotifier {
    fn new(events_proxy: glutin::EventsLoopProxy) -> WindowNotifier {
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

    fn new_frame_ready(
        &self,
        _doc_id: DocumentId,
        _scrolled: bool,
        _composite_needed: bool,
        _render_time: Option<u64>,
    ) {
        self.wake_up();
    }
}


struct Internals {
    gl_window: Option<glutin::WindowedContext<glutin::NotCurrent>>,
    events_loop: glutin::EventsLoop,
    font_store: Arc<Mutex<font::FontStore>>,
    api: RenderApi,
    document_id: DocumentId,
    pipeline_id: PipelineId,
    epoch: Epoch,
    renderer: webrender::Renderer,
    cursor_position: WorldPoint,
    dpi: f64,
    cursor_in_window: bool,
}

impl fmt::Debug for Internals {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Internals {{ window_id: {:?}, document_id: {:?}, pipeline_id: {:?}, cursor_position: {}, dpi: {} }}",
            self.get_window_id()/*gl_window.window().id()*/, self.document_id, self.pipeline_id, self.cursor_position, self.dpi )
    }
}

impl Internals {
    fn new(name: &str, width: f64, height: f64) -> Internals {
        let events_loop = glutin::EventsLoop::new();
        let window_builder = glutin::WindowBuilder::new()
            .with_title(name)
            .with_multitouch()
            .with_dimensions(glutin::dpi::LogicalSize::new(width, height));
        let window =
            glutin::ContextBuilder::new()
                .with_gl(glutin::GlRequest::GlThenGles {
                    opengl_version: (3, 2),
                    opengles_version: (3, 0),
                })
                .build_windowed(window_builder, &events_loop)
                .unwrap();

        let window = unsafe {
            window.make_current().unwrap()
        };

        let gl = match window.get_api() {
            glutin::Api::OpenGl => unsafe {
                gl::GlFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::OpenGlEs => unsafe {
                gl::GlesFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::WebGl => unimplemented!(),
        };

        let dpi = window.window().get_hidpi_factor();

        let opts = webrender::RendererOptions {
            device_pixel_ratio: dpi as f32,
            clear_color: Some(ColorF::new(0.2, 0.2, 0.2, 1.0)),
            //enable_scrollbars: true,
            //enable_aa:true,
            ..webrender::RendererOptions::default()
        };

        let framebuffer_size = {
            let size = window.window().get_inner_size().unwrap().to_physical(dpi);
            DeviceIntSize::new(size.width as i32, size.height as i32)
            //DeviceUintSize::new(size.width as u32, size.height as u32)
        };

        let notifier = Box::new(WindowNotifier::new(events_loop.create_proxy()));
        let (renderer, sender) =
            webrender::Renderer::new(gl.clone(), notifier, opts, None).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(framebuffer_size, 0);

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);

        let font_store = Arc::new(Mutex::new(font::FontStore::new(
            api.clone_sender().create_api(),
            document_id,
        )));

        let mut txn = Transaction::new();
        txn.set_root_pipeline(pipeline_id);
        api.send_transaction(document_id, txn);

        let window = unsafe{window.make_not_current().unwrap()};

        Internals {
            gl_window: Some(window),
            events_loop,
            font_store,
            api,
            document_id,
            pipeline_id,
            epoch,
            renderer,
            cursor_position: WorldPoint::new(0.0, 0.0),
            dpi,
            cursor_in_window: false,
        }
    }

    fn events(&mut self, tags: &[ItemTag]) -> Vec<PrimitiveEvent> {
        let mut events = Vec::new();

        let mut cursor_in_window = self.cursor_in_window;
        let mut cursor_position = self.cursor_position;
        let mut dpi = self.dpi;
        let mut txn = None;

        self.events_loop.poll_events(|event| {
            match event {
                glutin::Event::Awakened => return,
                _ => ()
            }
            //println!("event -> {:?}", &event);
            match event {
                glutin::Event::WindowEvent {
                    event: glutin::WindowEvent::CloseRequested,
                    window_id,
                } => {
                    {
                        TODEL.lock().unwrap().push(window_id);
                    }
                    events.push(PrimitiveEvent::Exit);
                }
                glutin::Event::WindowEvent {
                    event: glutin::WindowEvent::CursorEntered { .. },
                    window_id,
                } => {
                    //events.push(PrimitiveEvent::CursorEntered);
                    println!("window ID cursor entered ... {:?}", window_id);
                    cursor_in_window = true;
                }
                glutin::Event::WindowEvent {
                    event: glutin::WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    //println!(" >>>>>>>>>>>> cursor moved <<<<<<<<<<<< {}", cursor_in_window);
                    if cursor_in_window {
                        cursor_position = WorldPoint::new(position.x as f32, position.y as f32);
                        events.push(PrimitiveEvent::CursorMoved(position.into()));
                    }
                }
                glutin::Event::WindowEvent {
                    event: glutin::WindowEvent::CursorLeft { .. },
                    window_id,
                    ..
                } => {
                    //events.push(PrimitiveEvent::CursorLeft);

                    println!("window ID cursor left ... {:?}", window_id);
                    cursor_in_window = false;
                    cursor_position.x = -1.0;
                    cursor_position.y = -1.0;
                }
                glutin::Event::WindowEvent {
                    event: glutin::WindowEvent::Resized(size),
                    ..
                } => {
                    events.push(PrimitiveEvent::Resized(size));
                }
                glutin::Event::WindowEvent {
                    event: glutin::WindowEvent::HiDpiFactorChanged(factor),
                    ..
                } => {
                    dpi = factor;
                    events.push(PrimitiveEvent::DPI(factor));
                }
                glutin::Event::WindowEvent {
                    event:
                        glutin::WindowEvent::MouseInput {
                            state,
                            button,
                            modifiers,
                            ..
                        },
                    ..
                } => {
                    let _pos: properties::Position = cursor_position.into();
                    let _button = button.into();
                    let _state = state.into();
                    let _modifiers = modifiers.into();

                    if !tags.is_empty()
                        && button == glutin::MouseButton::Left
                        && state == glutin::ElementState::Released
                    {
                        events.push(PrimitiveEvent::SetFocus(true));
                    }
                    events.push(PrimitiveEvent::Button(_pos, _button, _state, _modifiers));
                }
                glutin::Event::WindowEvent {
                    event:
                        glutin::WindowEvent::MouseWheel {
                            delta, modifiers, ..
                        },
                    ..
                } => {
                    if txn.is_none() {
                        txn = Some(Transaction::new());
                    }
                    const LINE_HEIGHT: f32 = 38.0;
                    let (dx, dy) = if modifiers.alt {
                        match delta {
                            glutin::MouseScrollDelta::LineDelta(_, dy) => (dy * LINE_HEIGHT, 0.0),
                            glutin::MouseScrollDelta::PixelDelta(pos) => (pos.y as f32, 0.0),
                        }
                    } else {
                        match delta {
                            glutin::MouseScrollDelta::LineDelta(_, dy) => (0.0, dy * LINE_HEIGHT),
                            glutin::MouseScrollDelta::PixelDelta(pos) => (0.0, pos.y as f32),
                        }
                    };

                    if let Some(ref mut _txn) = txn {
                        _txn.scroll(
                            ScrollLocation::Delta(LayoutVector2D::new(dx, dy)),
                            cursor_position,
                        );
                    }

                    //println!("scrolling {} {}",dx,dy);
                }
                glutin::Event::WindowEvent {
                    event:
                        glutin::WindowEvent::KeyboardInput {
                            input:
                                glutin::KeyboardInput {
                                    scancode,
                                    state,
                                    virtual_keycode,
                                    modifiers,
                                },
                            ..
                        },
                    ..
                } => {
                    events.push(PrimitiveEvent::KeyInput(
                        virtual_keycode,
                        scancode,
                        state.into(),
                        modifiers.into(),
                    ));
                }
                glutin::Event::WindowEvent {
                    event: glutin::WindowEvent::ReceivedCharacter(c),
                    ..
                } => {
                    if c == '\x1b' {
                        events.push(PrimitiveEvent::SetFocus(false));
                    } else {
                        events.push(PrimitiveEvent::Char(c));
                    }
                }
                _ => (),
            }
        });

        self.dpi = dpi;

        if let Some(mut _txn) = txn {
            _txn.generate_frame();
            self.api.send_transaction(self.document_id, _txn);
        }

        self.cursor_in_window = cursor_in_window;
        self.cursor_position = cursor_position;

        events
    }

    fn get_window_id(&self) -> Option<glutin::WindowId> {
        match self.gl_window {
            None => None,
            Some(ref window) => Some(window.window().id()),
        }

    }

    fn deinit(self) {
        self.font_store.lock().unwrap().deinit();
        self.renderer.deinit();
    }
}

pub struct Window {
    width: f64,
    height: f64,
    root: Arc<Mutex<Element>>,
    name: String,
    id_generator: properties::IdGenerator,
    internals: Option<Internals>,
    tags: Vec<ItemTag>,
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Window {{ name: {}, width: {}, height: {}, internals: {:?} }}", self.name, self.width, self.height, self.internals)
    }
}

impl Window {
    pub fn new(root: Arc<Mutex<Element>>, name: String, width: f64, height: f64) -> Window {
        let id_generator = properties::IdGenerator::new(0);

        let mut _w = Window {
            width,
            height,
            root,
            name,
            id_generator,
            internals: None,
            tags: vec![],
        };

        _w.start_window();

        _w
    }

    fn start_window(&mut self) {
        self.internals = Some(Internals::new(&self.name, self.width, self.height));
    }

    fn get_tags(&mut self) -> (Vec<ItemTag>, Vec<ItemTag>) {
        let mut tags: Vec<ItemTag> = vec![];

        let mut new_tags: Vec<ItemTag> = vec![];
        let mut old_tags: Vec<ItemTag> = vec![];

        if let Some(ref mut i) = self.internals {
            if i.cursor_position.x > 0.0 && i.cursor_position.y > 0.0 {
                let results = i.api.hit_test(
                    i.document_id,
                    None,
                    i.cursor_position,
                    HitTestFlags::FIND_ALL,
                );
                let mut ind = results.items.len();
                while ind > 0 {
                    ind -= 1;
                    tags.push(results.items[ind].tag);
                }
            }
        }

        if self.tags.is_empty() {
            self.tags = tags.clone();
            new_tags = tags.clone();
        } else {
            for t in tags.iter() {
                let exists = self.tags.iter().find(|x| *x == t);
                if exists.is_none() {
                    new_tags.push(t.clone());
                }
            }

            for t in self.tags.iter() {
                let exists = tags.iter().find(|x| *x == t);
                if exists.is_none() {
                    old_tags.push(t.clone());
                }
            }

            self.tags = tags.clone();
        }

        if !new_tags.is_empty() || !old_tags.is_empty() {
            println! {"HoverBegin {:?}\nHoverEnd {:?}", new_tags, old_tags};
        }

        (new_tags, old_tags)
    }

    fn action_events(&mut self, events: Vec<PrimitiveEvent>, tags: &Vec<ItemTag>) {
        for e in events.iter() {
            /*if exit {
                return true;
            }*/
            match e {
                /*PrimitiveEvent::Exit => {
                    //exit = true;
                },*/
                PrimitiveEvent::CursorLeft => {
                    self.tags.clear();
                }
                PrimitiveEvent::Resized(size) => {
                    self.width = size.width;
                    self.height = size.height;
                }
                PrimitiveEvent::SetFocus(b) => {
                    if !*b {
                        self.root.lock().unwrap().on_primitive_event(&[], e.clone());
                    } else {
                        self.root
                            .lock()
                            .unwrap()
                            .on_primitive_event(&tags, e.clone());
                    }
                }
                PrimitiveEvent::Button(_, _, _, _) => {
                    self.root
                        .lock()
                        .unwrap()
                        .on_primitive_event(&tags, e.clone());
                }
                PrimitiveEvent::Char(_) => {
                    self.root
                        .lock()
                        .unwrap()
                        .on_primitive_event(&tags, e.clone());
                }
                PrimitiveEvent::CursorMoved(_) => {
                    self.root
                        .lock()
                        .unwrap()
                        .on_primitive_event(&tags, e.clone());
                }
                PrimitiveEvent::KeyInput(_, _, _, _) => {
                    self.root
                        .lock()
                        .unwrap()
                        .on_primitive_event(&tags, e.clone());
                }
                _ => (),
            }
        }
    }

    pub fn tick(&mut self) -> bool {
        let exit = false;

        let events;
        let mut dpi;
        let api;

        let (new_tags, old_tags) = self.get_tags();
        let tags = self.tags.clone();

        //collect events
        match self.internals {
            Some(ref mut i) => {
                events = i.events(&tags);
                dpi = i.dpi;
                api = i.api.clone_sender().create_api();
            },
            _ => panic!("in tick but no window internals initialized"),
        }

        if !new_tags.is_empty() {
            //events.insert(0,PrimitiveEvent::HoverBegin(new_tags));
            self.root
                .lock()
                .unwrap()
                .on_primitive_event(&[], PrimitiveEvent::HoverBegin(new_tags));
        }

        if !old_tags.is_empty() {
            //events.insert( 0,PrimitiveEvent::HoverEnd(old_tags));
            self.root
                .lock()
                .unwrap()
                .on_primitive_event(&[], PrimitiveEvent::HoverEnd(old_tags));
        }

        //if events.len() > 0 {
        //    println!("{:?}", events);
        //}

        self.action_events(events, &tags);

        let mut window: Option<glutin::WindowedContext<glutin::NotCurrent>> = None;

        match self.internals {
            Some(ref mut i) => {
                //begin
                std::mem::swap(&mut i.gl_window, &mut window);
            },
            _ => ()
        }

        let window = unsafe{window.unwrap().make_current().unwrap()};

        //let win_id = window.window().id();
        let mut txn = Transaction::new();
        let mut builder = None;
        let mut font_store = None;

        let (layout_size, framebuffer_size) = match self.internals {
            Some(ref mut i) => {
                dpi = window.window().get_hidpi_factor();
                let framebuffer_size = {
                    let size = window.window().get_inner_size().unwrap().to_physical(dpi);
                    DeviceIntSize::new(size.width as i32, size.height as i32)
                    //DeviceUintSize::new(size.width as u32, size.height as u32)
                };
                let layout_size = framebuffer_size.to_f32() / euclid::TypedScale::new(dpi as f32);

                builder = Some(DisplayListBuilder::new(i.pipeline_id, layout_size));

                font_store = Some(i.font_store.clone());

                (Some(layout_size), Some(framebuffer_size))
            },
            _ => (None, None)
        };

        let mut builder = builder.unwrap();
        let font_store = font_store.unwrap();
        let mut font_store = font_store.lock().unwrap();
        let font_store = font_store.deref_mut();
        let framebuffer_size = framebuffer_size.unwrap();
        let layout_size = layout_size.unwrap();

        self.render_root(&api, &mut builder, font_store, dpi as f32);

        if let Some(ref mut i) = self.internals {
            txn.set_window_parameters(
                framebuffer_size,
                DeviceIntRect::new(DeviceIntPoint::zero(), framebuffer_size),
                //DeviceUintRect::new(DeviceUintPoint::zero(), framebuffer_size),
                dpi as f32,
            );

            txn.set_display_list(i.epoch, None, layout_size, builder.finalize(), true);
            //txn.set_root_pipeline(i.pipeline_id);
            txn.generate_frame();
            i.api.send_transaction(i.document_id, txn);

            i.renderer.update();
            i.renderer.render(framebuffer_size).unwrap();
            let _ = i.renderer.flush_pipeline_info();
            window.swap_buffers().ok();
        }

        let mut window = unsafe{Some(window.make_not_current().unwrap())};
        match self.internals {
            Some(ref mut i) => {
                //Finally
                std::mem::swap(&mut i.gl_window, &mut window);
            },
            _ => ()
        }

        exit
    }

    fn render_root(
        &mut self,
        api: &RenderApi,
        builder: &mut DisplayListBuilder,
        font_store: &mut font::FontStore,
        dpi: f32,
    ) {
        let mut gen = self.id_generator.clone();
        gen.zero();

        let info = LayoutPrimitiveInfo::new((0.0, 0.0).by(self.width as f32, self.height as f32));
        builder.push_stacking_context(
            &info,
            None,
            TransformStyle::Flat,
            MixBlendMode::Normal,
            &[],
            RasterSpace::Screen,
        );

        self.root.lock().unwrap().render(
            api,
            builder,
            properties::Extent {
                x: 0.0,
                y: 0.0,
                w: self.width as f32,
                h: self.height as f32,
                dpi,
            },
            font_store,
            None,
            &mut gen,
        );

        builder.pop_stacking_context();
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let mut x = None;
        mem::swap(&mut x, &mut self.internals);
        let x = x.unwrap();
        x.deinit();
    }
}

lazy_static! {
    static ref TOADD: Mutex<Vec<(Arc<Mutex<Element>>, String, f64, f64)>> = Mutex::new(vec![]);
    static ref TODEL: Mutex<Vec<glutin::WindowId>> = Mutex::new(vec![]);
}

pub struct Manager {
    windows: Vec<Window>,
}

impl Manager {
    fn get() -> Option<Arc<Mutex<Manager>>> {
        static mut MANAGER: Option<Arc<Mutex<Manager>>> = None;

        unsafe {
            if MANAGER.is_none() {
                MANAGER = Some(Arc::new(Mutex::new(Manager { windows: vec![] })));
            }

            MANAGER.clone()
        }
    }

    pub fn start(fps: u64) {
        let fps = 1000 / fps;
        let mut lasttime;
        loop {
            lasttime = SystemTime::now();
            let mut i = 0;
            let mut wmo = Manager::get();
            if let Some(ref mut _wmo) = wmo {
                if let Ok(ref mut wm) = _wmo.lock() {
                    //add the windows to be added
                    if let Ok(ref mut to_add) = TOADD.lock() {
                        loop {
                            if to_add.len() > 0 {
                                let _t = to_add.remove(0);
                                wm.windows.push(Window::new(_t.0, _t.1, _t.2, _t.3));
                            } else {
                                break;
                            }
                        }
                    }
                    //render the windows
                    while i < wm.windows.len() {
                        /*if let Some(ref mut _int) = wm.windows[i].internals {
                            println!("Mem Rep : {:?} --> {:?}", _int.get_window_id(), _int.api.report_memory());//_int.api.report_memory()
                        }*/
                        wm.windows[i].tick();
                        i += 1;
                    }
                    //Remove Windows not required
                    if let Ok(ref mut to_del) = TODEL.lock() {

                        for wid in to_del.iter() {
                            let wid = wid.clone();
                            println!("Drop window ID {:?}, thread ID: {:?}", wid, thread::current().id());

                            for i in 0..wm.windows.len(){
                                if let Some(ref mut internal) = wm.windows[i].internals {
                                    if wid == internal.get_window_id().unwrap() {
                                        internal.api.shut_down();
                                        let x = wm.windows.remove(i);
                                        drop(x);
                                        println!("Window ID dropped {:?},, thread ID: {:?}", wid, thread::current().id());
                                        break;
                                    }
                                }
                            }
                        }
                        to_del.clear();
                        /*loop {
                            if to_del.len() > 0 {

                                println!("{:?}", wm.windows);

                                let wid = to_del.remove(0);
                                wm.windows.retain(|elm| {
                                    let mut keep = true;
                                    if let Some(ref mut int) = elm.internals {
                                        let _tid = int.get_window_id();
                                        if wid == _tid {
                                            keep = false;
                                        }
                                    }
                                    keep
                                });

                                println!("{:?}", wm.windows);
                            } else {
                                break;
                            }
                        }*/
                    }
                    //if all windows done, then exit the app
                    if wm.windows.is_empty() {
                        return;
                    }
                }
            }
            if let Ok(t) = lasttime.elapsed() {
                let mut t = u64::from(t.subsec_millis());
                if t > fps {
                    t = fps;
                }
                thread::sleep(Duration::from_millis(fps - t));
            } else {
                thread::sleep(Duration::from_millis(fps));
            }
        }
    }

    pub fn add(elem: Arc<Mutex<Element>>, name: String, width: f64, height: f64) {
        if let Ok(ref mut to_add) = TOADD.lock() {
            to_add.push((elem, name, width, height));
        }
    }
}
