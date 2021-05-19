use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

use shaderc::{Compiler, ShaderKind};

gears_pipeline::pipeline! {
    vert: {
        path: "src/shader/vert.glsl"
    }
}

pub const DEFAULT_FRAG_REF: &str = include_str!("shader/frag.glsl");

fn create_shader(path: PathBuf) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(DEFAULT_FRAG_REF.as_bytes())?;
    Ok(())
}

fn try_read_shader(path: PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    Ok(buf)
}

pub fn read_shader(path: PathBuf) -> Result<Vec<u8>, shaderc::Error> {
    let mut shaderc = Compiler::new().unwrap();

    let shader_source = try_read_shader(path.clone());

    let shader_source = if let Ok(s) = shader_source.as_ref() {
        s.as_ref()
    } else {
        create_shader(path.clone()).unwrap();
        DEFAULT_FRAG_REF
    };

    shaderc
        .compile_into_spirv(
            shader_source,
            ShaderKind::Fragment,
            path.file_name().unwrap().to_str().unwrap(),
            "main",
            None,
        )
        .map(|artifact| artifact.as_binary_u8().into())
}
