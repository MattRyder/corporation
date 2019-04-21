use shaderc;

pub enum Kind {
    Vertex,
    Fragment,
}

#[derive(Debug)]
pub enum Error {
    CompileError(String),
}

pub struct Loader {}

impl Loader {
    pub fn compile<'a>(name: &str, kind: &Kind, shader_source: &str) -> Result<Vec<u8>, Error> {
        let mut compiler = shaderc::Compiler::new().unwrap();
        let opts = shaderc::CompileOptions::new().unwrap();

        let shader_kind = match kind {
            Kind::Fragment => shaderc::ShaderKind::Fragment,
            Kind::Vertex => shaderc::ShaderKind::Vertex,
        };

        match compiler.compile_into_spirv(shader_source, shader_kind, name, "main", Some(&opts)) {
            Ok(compilation_artifact) => Ok(compilation_artifact.as_binary_u8().to_vec()),
            Err(error) => Err(Error::CompileError(format!(
                "Failed to compile shader '{}':\nLog: {}",
                &name, &error
            ))),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    const NAME : &str = "test_vs_shader";

    #[test]
    fn should_compile_shader() {
        let source = "#version 150 core\n void main() {}";

        let result = Loader::compile(&NAME, &Kind::Vertex, &source);

        assert!(result.is_ok());
    }

    #[test]
    fn should_raise_compile_error() {
        let source = "";
        let result = Loader::compile(&NAME, &Kind::Vertex, &source);

        assert!(result.is_err());
    }
}
