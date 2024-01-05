use wgpu::{PipelineLayoutDescriptor, RenderPipelineDescriptor};

fn main(){
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let primary_size = event_loop.primary_monitor().unwrap().size();
    // sure hope nobody has a monitor less than 10 pixels 
    let width = (primary_size.width / 6).max(1);
    let height = (primary_size.height / 6).max(1);
    let window = winit::window::WindowBuilder::new()
        .with_min_inner_size(winit::dpi::PhysicalSize{width,height,})
        .with_title("click within inner to toggle window decorations")
        .build(&event_loop)
        .unwrap();
    env_logger::init();
    pollster::block_on(run(event_loop, window));
}


async fn run(event_loop: winit::event_loop::EventLoop<()>, window: winit::window::Window) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);
    let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions{
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }).await.expect("couldn't find an adapter");
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::downlevel_defaults()
                .using_resolution(adapter.limits())
        },
        None
    ).await.expect("couldn't find an adequate device");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl")))
    });
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];
    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor{
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[]
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())]
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });
    let mut config = wgpu::SurfaceConfiguration{
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    event_loop.run(
    move |event, target|{
        // so they get cleaned up since run() never returns
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        if let winit::event::Event::WindowEvent {
            event, // shadowing
            window_id: _
        } = event {
        match event {
            winit::event::WindowEvent::Resized(new_size) => {
                config.width = new_size.width.max(1);
                config.height = new_size.height.max(1);
                surface.configure(&device, &config);
                window.request_redraw();
            }
            winit::event::WindowEvent::RedrawRequested =>{
                let frame = surface
                    .get_current_texture()
                    .expect("coulnd't get next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});
                {
                    let mut rpass =
                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                    store: wgpu::StoreOp::Store,
                                }
                            })],
                            ..Default::default()
                        });
                        rpass.set_pipeline(&render_pipeline);
                        rpass.draw(0..3, 0..1);
                }
                queue.submit(Some(encoder.finish()));
                frame.present();
            },
            winit::event::WindowEvent::CloseRequested => target.exit(),
            winit::event::WindowEvent::MouseInput {state, ..} => {
                if state.is_pressed(){
                    window.set_decorations(!window.is_decorated());
                }
            },
            _ => (),
            }
        }
    })
    .unwrap();
}
