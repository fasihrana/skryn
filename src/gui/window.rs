
use gleam::gl;
use glutin;
use glutin::GlContext;
use winit;
use webrender;
use webrender::api::*;
use euclid;

use elements::element;
use gui::font;
use gui::properties;

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
        }
    }

    pub fn deinit(self){
        let x = self.renderer;
        x.deinit();
    }

    fn render (&mut self){
        if let Some(ref mut r) = self.root {
            unsafe {self.window.make_current()};

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

            /*txn.set_window_parameters(DeviceUintSize::new(_width as u32, _height as u32),
                                      DeviceUintRect::new(euclid::TypedPoint2D::new(0 as u32,0 as u32),
                                      euclid::TypedSize2D::new(_width as u32, _height as u32)),
                                      device_pixel_ratio);*/

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

    pub fn tick(&mut self) -> bool {
        let mut do_exit = false;

        self.events_loop.poll_events(|event|{
            match event {
                winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. /*window_id*/ } => {
                    do_exit = true;
                },
                _ => ()
            }
        });

        if !do_exit {
            self.render();
        }

        return do_exit;
    }

    pub fn set_root(&mut self, r: Box<element::Element> ){
        self.root = Some(r);
    }
}