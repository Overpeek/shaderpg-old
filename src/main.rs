use gears::{
    glam::{Vec2, Vec4},
    Buffer, EventLoopTarget, Frame, FrameLoop, FrameLoopTarget, FramePerfReport,
    ImmediateFrameInfo, IndexBuffer, KeyboardInput, RenderRecordBeginInfo, RenderRecordInfo,
    Renderer, RendererRecord, SyncMode, UpdateLoop, UpdateLoopTarget, UpdateRate, UpdateRecordInfo,
    VertexBuffer, VirtualKeyCode, WindowEvent,
};
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

struct App {
    renderer: Renderer,

    ib: IndexBuffer<u16>,
    vb: VertexBuffer<shader::VertexData>,

    shader: shader::Pipeline, // active shader
    discarded_shaders: Vec<shader::Pipeline>,

    main_shader_in_use: AtomicUsize,
    last_modified: SystemTime,

    dt: Instant,
}

impl App {
    fn new(renderer: Renderer) -> Arc<RwLock<Self>> {
        let vertices: &[shader::VertexData; 4] = &[
            shader::VertexData {
                pos: Vec2::new(-SCALE, -SCALE),
            },
            shader::VertexData {
                pos: Vec2::new(SCALE, -SCALE),
            },
            shader::VertexData {
                pos: Vec2::new(SCALE, SCALE),
            },
            shader::VertexData {
                pos: Vec2::new(-SCALE, SCALE),
            },
        ];

        let ib = IndexBuffer::new_with_data(&renderer, INDICES).unwrap();
        let vb = VertexBuffer::new_with_data(&renderer, vertices).unwrap();

        let shader = shader::Pipeline::build(&renderer).unwrap();

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
        match shader::Pipeline::build(&self.renderer) {
            Ok(mut discarded_shader) => {
                self.main_shader_in_use.store(0, Ordering::SeqCst);

                mem::swap(&mut self.shader, &mut discarded_shader);
                self.discarded_shaders.push(discarded_shader);
                self.renderer.request_rerecord();

                log::info!("Shader reloaded");
            }
            Err(compile_error) => log::error!("Shader build failed: {}", compile_error),
        }
    }
}

impl UpdateLoopTarget for App {
    fn update(&mut self, _: &Duration) {
        let metadata = fs::metadata("shader.glsl").unwrap();

        let modified = metadata.modified().unwrap();
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
        let ubo = shader::UniformData {
            time: self.dt.elapsed().as_secs_f32(),
        };
        self.shader.write_fragment_uniform(imfi, &ubo).unwrap();
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
            clear_color: Vec4::new(1.0, 0.0, 0.5, 1.0),
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
        .with_rate(UpdateRate::PerSecond(10))
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
