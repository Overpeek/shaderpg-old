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
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

mod shader;

const FRAMES_IN_FLIGHT: usize = 3;
const SCALE: f32 = 1.0;
const INDICES: &[u16; 6] = &[0, 1, 2, 0, 2, 3];

struct ROs {
    ib: IndexBuffer<u16>,
    vb: VertexBuffer<shader::VertexData>,

    shader: shader::Pipeline, // active shader
    discarded_shaders: Vec<shader::Pipeline>,

    main_shader_in_use: usize,
}

struct App {
    frame: Frame,
    renderer: Renderer,
    ros: RwLock<ROs>,

    last_modified: RwLock<SystemTime>,
    dt: RwLock<Instant>,
}

impl App {
    fn new(frame: Frame, renderer: Renderer) -> Arc<RwLock<Self>> {
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

        let ib = IndexBuffer::new_with(&renderer, INDICES).unwrap();
        let vb = VertexBuffer::new_with(&renderer, vertices).unwrap();

        let shader = shader::Pipeline::build(&renderer).unwrap();

        let ros = RwLock::new(ROs {
            ib,
            vb,

            shader,
            discarded_shaders: Vec::new(),

            main_shader_in_use: 3,
        });

        let last_modified = RwLock::new(SystemTime::now());
        let dt = RwLock::new(Instant::now());

        let app = Self {
            frame,
            renderer,
            ros,

            last_modified,
            dt,
        };

        Arc::new(RwLock::new(app))
    }

    fn update_shader(&self) {
        match shader::Pipeline::build(&self.renderer) {
            Ok(mut discarded_shader) => {
                let mut ros = self.ros.write();
                ros.main_shader_in_use = 0;

                mem::swap(&mut ros.shader, &mut discarded_shader);
                ros.discarded_shaders.push(discarded_shader);
                self.renderer.request_rerecord();

                log::info!("Shader reloaded");
            }
            Err(compile_error) => log::error!("Shader build failed: {}", compile_error),
        }
    }
}

impl UpdateLoopTarget for App {
    fn update(&self, _: &Duration) {
        let metadata = fs::metadata("shader.glsl").unwrap();

        let modified = metadata.modified().unwrap();
        let mut last_modified = self.last_modified.write();
        if modified > *last_modified {
            *last_modified = modified;
            self.update_shader();
        }

        let mut ros = self.ros.write();
        if ros.main_shader_in_use >= FRAMES_IN_FLIGHT {
            ros.discarded_shaders.clear();
        }
    }
}

impl EventLoopTarget for App {
    fn event(&self, event: &WindowEvent) {
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
            time: self.dt.read().elapsed().as_secs_f32(),
            aspect: self.frame.aspect(),
        };

        let ros = self.ros.read();
        ros.shader.write_fragment_uniform(imfi, &ubo).unwrap();
    }

    unsafe fn update(&self, uri: &UpdateRecordInfo) -> bool {
        let mut ros = self.ros.write();
        [
            ros.shader.update(uri),
            ros.vb.update(uri),
            ros.ib.update(uri),
        ]
        .iter()
        .any(|b| *b)
    }

    fn begin_info(&self) -> RenderRecordBeginInfo {
        RenderRecordBeginInfo {
            clear_color: Vec4::new(1.0, 0.0, 0.5, 1.0),
            debug_calls: false,
        }
    }

    unsafe fn record(&self, rri: &RenderRecordInfo) {
        let mut ros = self.ros.write();
        ros.main_shader_in_use = 1;
        ros.shader
            .draw(rri)
            .vertex(&ros.vb)
            .index(&ros.ib)
            .direct(ros.ib.elem_capacity() as u32, 0)
            .execute();
    }
}

impl FrameLoopTarget for App {
    fn frame(&self) -> FramePerfReport {
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

    let app = App::new(frame, renderer);

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
