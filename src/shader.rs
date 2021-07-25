use std::{
    fs::File,
    io::{self, Read, Write},
};

use gears::{glam::Vec2, module, pipeline, FormatOf, Input, RGBAOutput, Uniform};

#[derive(Input, Default)]
pub struct VertexData {
    pub pos: Vec2,
}

#[derive(Uniform, Default)]
pub struct UniformData {
    pub time: f32,
}

module! {
    kind = "vert",
    path = "src/shader/vert.glsl",
    name = "VERT"
}

module! {
    kind = "frag",
    path = "src/shader/frag.glsl",
    name = "FRAG",
    runtime = "reader"
}

pipeline! {
    "Pipeline"
    VertexData -> RGBAOutput

    mod "VERT" as "vert"
    mod "FRAG" as "frag" where { in UniformData }
}

pub const DEFAULT_FRAG_REF: &'static str = include_str!("shader/frag.glsl");
pub const PATH: &'static str = "shader.glsl";

fn create_shader() -> io::Result<()> {
    let mut file = File::create(PATH)?;
    file.write_all(DEFAULT_FRAG_REF.as_bytes())?;
    Ok(())
}

fn try_read_shader() -> io::Result<String> {
    let mut file = File::open(PATH)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    Ok(buf)
}

pub fn reader(_: &'static str) -> String {
    match try_read_shader() {
        Ok(s) => s,
        Err(_) => {
            create_shader().unwrap();
            try_read_shader().unwrap()
        }
    }
}
