use cgmath::{Vector2, Vector4};
use gears::{
    Buffer, Frame, FrameLoop, FrameLoopTarget, FramePerfReport, ImmediateFrameInfo, Pipeline,
    PipelineBuilder, RenderRecordBeginInfo, RenderRecordInfo, Renderer, RendererRecord,
    UpdateRecordInfo, VertexBuffer,
};
use log::error;
use parking_lot::RwLock;
use std::sync::Arc;

mod shader;

const VERTICES: &[shader::VertexData; 6] = &[
    // tri a
    shader::VertexData {
        pos: Vector2::new(-1.0, -1.0),
    },
    shader::VertexData {
        pos: Vector2::new(1.0, -1.0),
    },
    shader::VertexData {
        pos: Vector2::new(1.0, 1.0),
    },
    // tri b
    shader::VertexData {
        pos: Vector2::new(-1.0, -1.0),
    },
    shader::VertexData {
        pos: Vector2::new(1.0, 1.0),
    },
    shader::VertexData {
        pos: Vector2::new(-1.0, 1.0),
    },
];

struct App {
    renderer: Renderer,

    vb: VertexBuffer<shader::VertexData>,
    shader: Pipeline,               // active shader
    blend_shader: Option<Pipeline>, // blending shader
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

        let vb = VertexBuffer::new_with_data(&renderer, VERTICES).unwrap();
        let shader = PipelineBuilder::new(&renderer)
            .with_graphics_modules(shader::VERT_SPIRV_REF, &shader_source[..])
            .with_ubo::<shader::UBO>()
            .with_input::<shader::VertexData>()
            .build(false)
            .unwrap();

        let app = Self {
            renderer,

            vb,
            shader,
            blend_shader: None,
        };

        Arc::new(RwLock::new(app))
    }
}

impl RendererRecord for App {
    fn immediate(&self, imfi: &ImmediateFrameInfo) {
        let ubo = shader::UBO { time: 0.0 };
        self.shader.write_ubo(imfi, &ubo).unwrap();
    }

    fn update(&self, uri: &UpdateRecordInfo) -> bool {
        unsafe { self.shader.update(uri) || self.vb.update(uri) }
    }

    fn begin_info(&self) -> RenderRecordBeginInfo {
        RenderRecordBeginInfo {
            clear_color: Vector4::new(0.0, 0.0, 0.0, 1.0),
            debug_calls: true,
        }
    }

    fn record(&self, rri: &RenderRecordInfo) {
        unsafe {
            self.shader.bind(rri);
            self.vb.draw(rri);
        }
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

    let renderer = Renderer::new().build(context).unwrap();

    let app = App::new(renderer);

    FrameLoop::new()
        .with_event_loop(event_loop)
        .with_frame_target(app)
        .build()
        .run();
}
