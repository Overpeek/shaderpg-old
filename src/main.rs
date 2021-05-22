use cgmath::{Vector2, Vector4};
use gears::{
    Buffer, EventLoopTarget, Frame, FrameLoop, FrameLoopTarget, FramePerfReport,
    ImmediateFrameInfo, IndexBuffer, KeyboardInput, Pipeline, PipelineBuilder,
    RenderRecordBeginInfo, RenderRecordInfo, Renderer, RendererRecord, SyncMode, UpdateLoop,
    UpdateLoopTarget, UpdateRate, UpdateRecordInfo, VertexBuffer, VirtualKeyCode, WindowEvent,
};
use log::*;
use parking_lot::RwLock;
use std::{
    fs, mem,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};

mod shader;

const FRAMES_IN_FLIGHT: usize = 3;
const SCALE: f32 = 1.0;
const INDICES: &[u16; 6] = &[0, 1, 2, 0, 2, 3];
const VERTICES: &[shader::VertexData; 4] = &[
    shader::VertexData {
        pos: Vector2::new(-SCALE, -SCALE),
    },
    shader::VertexData {
        pos: Vector2::new(SCALE, -SCALE),
    },
    shader::VertexData {
        pos: Vector2::new(SCALE, SCALE),
    },
    shader::VertexData {
        pos: Vector2::new(-SCALE, SCALE),
    },
];

struct App {
    renderer: Renderer,

    ib: IndexBuffer<u16>,
    vb: VertexBuffer<shader::VertexData>,

    shader: Pipeline, // active shader
    discarded_shaders: Vec<Pipeline>,

    main_shader_in_use: AtomicUsize,
    last_modified: SystemTime,

    dt: Instant,
}

impl App {
    fn new(renderer: Renderer) -> Arc<RwLock<Self>> {
        let shader_source =
            shader::read_shader("shader.glsl".into()).unwrap_or_else(|err| match err {
                shaderc::Error::CompilationError(count, message) => {
                    error!("Compilation errors: {}\nMessage: {}", count, message);
                    panic!()
                }
                err => panic!("Shaderc error: {:?}", err),
            });

        let ib = IndexBuffer::new_with_data(&renderer, INDICES).unwrap();
        let vb = VertexBuffer::new_with_data(&renderer, VERTICES).unwrap();
        let shader = PipelineBuilder::new(&renderer)
            .with_graphics_modules(shader::VERT_SPIRV_REF, &shader_source[..])
            .with_ubo::<shader::UBO>()
            .with_input::<shader::VertexData>()
            .build(false)
            .unwrap();

        let app = Self {
            renderer,

            ib,
            vb,
            shader,
            discarded_shaders: Vec::new(),

            main_shader_in_use: AtomicUsize::new(3),
            last_modified: SystemTime::now(),

            dt: Instant::now(),
        };

        Arc::new(RwLock::new(app))
    }

    fn update_shader(&mut self) {
        match shader::read_shader("shader.glsl".into()) {
            Ok(shader_source) => {
                match PipelineBuilder::new(&self.renderer)
                    .with_graphics_modules(shader::VERT_SPIRV_REF, &shader_source[..])
                    .with_ubo::<shader::UBO>()
                    .with_input::<shader::VertexData>()
                    .build(false)
                {
                    Ok(mut discarded_shader) => {
                        self.main_shader_in_use.store(0, Ordering::SeqCst);

                        mem::swap(&mut self.shader, &mut discarded_shader);
                        self.discarded_shaders.push(discarded_shader);
                        self.renderer.request_rerecord();
                    }
                    Err(err) => println!("Shader error: {:?}", err),
                }
            }
            Err(compile_error) => {
                println!("\n\nShader compile error: {}", compile_error);
            }
        }
    }
}

impl UpdateLoopTarget for App {
    fn update(&mut self, _: &Duration) {
        let modified = fs::metadata("shader.glsl").unwrap().modified().unwrap();
        if modified > self.last_modified {
            self.last_modified = modified;
            self.update_shader();
        }

        if self.main_shader_in_use.load(Ordering::SeqCst) >= FRAMES_IN_FLIGHT {
            self.discarded_shaders.clear();
        }
    }
}

impl EventLoopTarget for App {
    fn event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => self.update_shader(),
            _ => {}
        }
    }
}

impl RendererRecord for App {
    fn immediate(&self, imfi: &ImmediateFrameInfo) {
        let ubo = shader::UBO {
            time: self.dt.elapsed().as_secs_f32(),
        };
        self.shader.write_ubo(imfi, &ubo).unwrap();
    }

    fn update(&self, uri: &UpdateRecordInfo) -> bool {
        unsafe {
            let a = self.shader.update(uri);
            let b = self.vb.update(uri);
            let c = self.ib.update(uri);
            a || b || c
        }
    }

    fn begin_info(&self) -> RenderRecordBeginInfo {
        RenderRecordBeginInfo {
            clear_color: Vector4::new(1.0, 0.0, 0.5, 1.0),
            debug_calls: false,
        }
    }

    fn record(&self, rri: &RenderRecordInfo) {
        unsafe {
            self.main_shader_in_use.fetch_add(1, Ordering::SeqCst);

            self.shader.bind(rri);
            self.ib.draw(rri, &self.vb);
        }
    }
}

impl FrameLoopTarget for App {
    fn frame(&self) -> FramePerfReport {
        // self.renderer.request_rerecord();
        self.renderer.frame(self)
    }
}

fn main() {
    env_logger::init();

    let (frame, event_loop) = Frame::new()
        .with_title("ShaderPG")
        .with_size(600, 600)
        .with_min_size(64, 64)
        .build();

    let context = frame.default_context().unwrap();

    let renderer = Renderer::new()
        .with_sync(SyncMode::Immediate)
        .with_frames_in_flight(FRAMES_IN_FLIGHT)
        .build(context)
        .unwrap();

    let app = App::new(renderer);

    let _ = UpdateLoop::new()
        .with_rate(UpdateRate::PerSecond(2))
        .with_target(app.clone())
        .build()
        .run();

    FrameLoop::new()
        .with_event_loop(event_loop)
        .with_frame_target(app.clone())
        .with_event_target(app)
        .build()
        .run();
}
