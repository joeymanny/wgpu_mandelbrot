

use wgpu::{PipelineLayoutDescriptor, RenderPipelineDescriptor, util::DeviceExt};
use winit::{event::{MouseButton, KeyEvent}, keyboard::{KeyCode, Key, PhysicalKey}};
fn main(){
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let primary_size = event_loop.primary_monitor().unwrap().size();
    // sure hope nobody has a monitor less than 10 pixels 
    let size = std::cmp::min(primary_size.width, primary_size.height);
    let window = winit::window::WindowBuilder::new()
        // .with_min_inner_size(winit::dpi::PhysicalSize{width,height,})
        .with_inner_size(winit::dpi::PhysicalSize::new(size,size))
        .with_title("mandelbrot set")
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
    let mandel_commands: &mut [f32; 6] = &mut [
        size.width as f32, size.height as f32, // screen dimensions
        0.0, 0.0, // linear offset
        2.0, // zoom
        300.0, // level of detail (num mandel iterations)
    ];
    let staging_mandel_commands_buffer = device.create_buffer(&wgpu::BufferDescriptor{
        label: None,
        size: std::mem::size_of::<f32>() as u64 * 6,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
        mapped_at_creation: false,
    });
    let mandel_commands_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: None,
        contents: bytemuck::cast_slice(mandel_commands),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST

    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry{
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer{
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry{
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding{
                buffer: &mandel_commands_buffer,
                offset: 0,
                size: None,
            })
        }]
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl")))
    });
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
        label: None,
        bind_group_layouts: &[&bind_group_layout],
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
        let mut is_mandel_update = false;
        if let winit::event::Event::WindowEvent {
            event, // shadowing
            window_id: _
        } = event {
        match event {
            winit::event::WindowEvent::Resized(new_size) => {
                println!("resize");

                config.width = new_size.width.max(1);
                config.height = new_size.height.max(1);
                surface.configure(&device, &config);

                mandel_commands[0] = new_size.width as f32;
                mandel_commands[1] = new_size.height as f32;
                is_mandel_update = true;

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
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                                    store: wgpu::StoreOp::Store,
                                }
                            })],
                            ..Default::default()
                        });
                        rpass.set_bind_group(0, &bind_group, &[]);
                        rpass.set_pipeline(&render_pipeline);
                        rpass.draw(0..3, 0..2);
                }
                queue.submit(Some(encoder.finish()));
                frame.present();
            },
            winit::event::WindowEvent::CloseRequested => target.exit(),
            winit::event::WindowEvent::MouseWheel {delta, ..} => {
                    if let winit::event::MouseScrollDelta::LineDelta(_, y) = delta{
                        if y.is_sign_positive(){
                            is_mandel_update = true;
                            mandel_commands[4] -= 1.0;
                            window.request_redraw();
                        }else if y.is_sign_negative(){
                            is_mandel_update = true;
                            mandel_commands[4] = (mandel_commands[4] + 1.0).min(2.0);
                            window.request_redraw();
                        }
                    }
            },
            winit::event::WindowEvent::KeyboardInput {event, ..}=>{

                if let KeyEvent{physical_key: winit::keyboard::PhysicalKey::Code(key), state: winit::event::ElementState::Pressed, ..} = event{
                    let mut delta_offset: (i8, i8) = (0, 0);
                    if let KeyCode::ArrowLeft | KeyCode::KeyA = key{
                        delta_offset.0 += -1;
                    }
                    if let KeyCode::ArrowRight | KeyCode::KeyD = key{
                        delta_offset.0 += 1;
                    }
                    if let KeyCode::ArrowUp | KeyCode::KeyW = key{
                        delta_offset.1 += -1; // 0,0 is at top-left
                    }
                    if let KeyCode::ArrowDown | KeyCode::KeyS = key{
                        delta_offset.1 += 1;
                    }
                    if delta_offset != (0, 0){
                        is_mandel_update = true;
                        mandel_commands[2] += (2.0f32.powf(mandel_commands[4]) / 4.0) * delta_offset.0 as f32;
                        mandel_commands[3] += (2.0f32.powf(mandel_commands[4]) / 4.0) * delta_offset.1 as f32;
                        window.request_redraw();
                    }
                    let mut delta_lod: i8 = 0;
                    if let KeyCode::Equal = key{
                        delta_lod += 1;
                    }
                    if let KeyCode::Minus = key{
                        delta_lod += -1;
                    }
                    if delta_lod != 0 {
                        mandel_commands[5] *= 2.0_f32.powi(delta_lod.into());
                        is_mandel_update = true;
                        window.request_redraw();
                    }
                }
            },
            _ => (),
            }
        }
        if is_mandel_update{
            dbg!(&mandel_commands);
            let (sender, receiver) = flume::bounded(1);
            let slice = staging_mandel_commands_buffer.slice(..);
            slice.map_async(wgpu::MapMode::Write, move |v| sender.send(v).unwrap());
            device.poll(wgpu::Maintain::Wait);

            if let Ok(Ok(())) = receiver.recv(){
                let mut mapped = slice.get_mapped_range_mut();
                mapped.clone_from_slice(bytemuck::cast_slice(mandel_commands));
                drop(mapped);
                staging_mandel_commands_buffer.unmap();
                let mut encoder = device.create_command_encoder(&Default::default());
                encoder.copy_buffer_to_buffer(
                    &staging_mandel_commands_buffer, 0,
                    &mandel_commands_buffer, 0,
                    std::mem::size_of::<f32>() as u64 * 6,
                );
                queue.submit(Some(encoder.finish()));
            }
        }
    })
    .unwrap();
}
