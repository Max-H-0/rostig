use wgpu::{wgc::device, wgt::DeviceDescriptor};
use winit::{event::{self, *}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};
use winit::window::Window;


struct State<'a>
{
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window
}

impl<'a> State<'a>  
{
    async fn new(window: &'a Window) -> State<'a>
    {
        let size = window.inner_size();


        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor 
        { 
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions
        { 
            power_preference: wgpu::PowerPreference::default(), 
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor
        {
            required_features: wgpu::Features::empty(),
            required_limits: if (!cfg!(target_arch="wasm32")) { 
                wgpu::Limits::downlevel_webgl2_defaults() 
            } else { 
                wgpu::Limits::default() 
            },
            label: None,
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off
        }).await.unwrap();

        let capabilities = surface.get_capabilities(&adapter);

        let surgace_format = capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);
            
        let config = wgpu::SurfaceConfiguration
        {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surgace_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, //capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };


        Self
        {
            surface,
            device,
            queue,
            config,
            size,
            window
        }
    }


    pub fn window (&self) -> &Window 
    {
        &self.window
    }


    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
    {
        if new_size.width > 0 && new_size.height > 0
        {
            self.size = new_size;

            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.surface.configure(&self.device, &self.config);
        }
    } 

    fn input(&mut self, event: &WindowEvent) -> bool 
    {
        false
    }

    fn update(&mut self)
    {
        
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor
        {
            label: Some("Render Encoder")
        });

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor 
        { 
            label: Some("Render Pass"), 
            color_attachments: &[Some(wgpu::RenderPassColorAttachment 
            {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations 
                {
                    load: wgpu::LoadOp::Clear(wgpu::Color 
                    { 
                        r: 0.1, 
                        g: 0.2, 
                        b: 0.3, 
                        a: 1.0 
                    }),
                    store: wgpu::StoreOp::Store
                }
            })], 
            depth_stencil_attachment: None, 
            occlusion_query_set: None,
            timestamp_writes: None
        });
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();


        Ok(())
    }
}


pub async fn run()
{
    cfg_if::cfg_if! 
    {
        if #[cfg(target_arch = "wasm32")] 
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } 
        else 
        {
            env_logger::init();
        }
    }

 
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    
    #[cfg(target_arch = "wasm32")] 
    {
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));
        
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| 
            {
                let dst = doc.get_element_by_id("rostig")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                
                dst.append_child(&canvas).ok()?;

                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    
    let mut state = State::new(&window).await;
    let mut surface_configured = false;


    let result = event_loop.run(move |event, control_flow| match event 
    {
        Event::WindowEvent{ref event, window_id} =>
        {
            if window_id == state.window().id() && !state.input(event)
            {
                match event  
                {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput 
                    {
                        event:  KeyEvent 
                        {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                        ..
                    } => control_flow.exit(),

                    WindowEvent::Resized(physical_size) =>
                    {
                        state.resize(*physical_size);
                        surface_configured = true;
                    }

                    WindowEvent::RedrawRequested =>
                    {
                        state.window().request_redraw();


                        if !surface_configured { return; }


                        state.update();
                        match state.render()
                        {
                            Ok(_) => {}

                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)
                                => state.resize(state.size),

                            Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) =>
                            {
                                log::error!("OutOfMemory");
                                control_flow.exit();
                            },

                            Err(wgpu::SurfaceError::Timeout) =>
                            {
                                log::warn!("Surface timeout");
                            }
                        }
                    }

                    _ => {}
                }
            }
        },
        
        _ => {}
    });
    

    match result 
    {
        Ok(_) => println!("Exited without error."),
        Err(error) => println!("Exited with error: {}", error),
    }
}
