use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
};

pub struct Renderer {
    instance: wgpu::Instance,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<wgpu::RenderPipeline>,
    _window: Option<ndk::native_window::NativeWindow>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            // wgpu 24: Instance::new takes &InstanceDescriptor
            instance: wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
                ..Default::default()
            }),
            surface: None,
            device: None,
            queue: None,
            config: None,
            pipeline: None,
            _window: None,
        }
    }

    pub fn attach_surface(&mut self, window: ndk::native_window::NativeWindow) {
        self.detach_surface();

        let width = window.width() as u32;
        let height = window.height() as u32;

        // raw-window-handle 0.6: AndroidDisplayHandle::new() replaces ::empty()
        let raw_display = RawDisplayHandle::Android(AndroidDisplayHandle::new());

        // raw-window-handle 0.6: AndroidNdkWindowHandle::new takes NonNull<c_void>
        let anw_ptr: std::ptr::NonNull<std::ffi::c_void> = window.ptr().cast();
        let raw_window = RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(anw_ptr));
        
        let target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: raw_display,
            raw_window_handle: raw_window,
        };

        let surface = unsafe { self.instance.create_surface_unsafe(target) }
            .expect("create_surface_unsafe failed");

        // NOTE: adapter must exist before querying capabilities.
        let adapter = pollster::block_on(self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .expect("No compatible adapter");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let (device, queue) = pollster::block_on(
            adapter.request_device(&wgpu::DeviceDescriptor::default(), None),
        )
        .expect("Failed to create device");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                @vertex
                fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
                    var p = array<vec2f, 3>(
                        vec2f(0.0, 0.6),
                        vec2f(-0.6, -0.4),
                        vec2f(0.6, -0.4),
                    );
                    return vec4f(p[i], 0.0, 1.0);
                }

                @fragment
                fn fs() -> @location(0) vec4f {
                    return vec4f(1.0, 0.45, 0.15, 1.0);
                }
                "#
                .into(),
            ),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.pipeline = Some(pipeline);
        self._window = Some(window);
    }

    pub fn detach_surface(&mut self) {
        self.pipeline = None;
        self.surface = None;
        self.config = None;
        self._window = None;
        self.device = None;
        self.queue = None;
    }

    pub fn render_frame(&mut self, _time: f64) {
        let (Some(surface), Some(device), Some(queue), Some(config), Some(pipeline)) = (
            &self.surface,
            &self.device,
            &self.queue,
            &self.config,
            &self.pipeline,
        ) else {
            return;
        };

        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                surface.configure(device, config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Surface out of memory");
                return;
            }
            // wgpu 24 added SurfaceError::Other — handle it explicitly.
            Err(wgpu::SurfaceError::Timeout) | Err(wgpu::SurfaceError::Other) => return,
        };

        let view = output.texture.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.04,
                            g: 0.05,
                            b: 0.10,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipeline);
            pass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        output.present();
    }
}
