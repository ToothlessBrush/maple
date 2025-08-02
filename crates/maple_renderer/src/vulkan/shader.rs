use std::sync::Arc;

use anyhow::Result;
use anyhow::anyhow;
use naga::valid::Capabilities;
use naga::valid::ValidationFlags;
use naga::valid::Validator;
use vulkano::shader::ShaderModule;

use naga::{
    back::spv::{self, PipelineOptions},
    front::glsl::{Frontend, Options},
    valid::ModuleInfo,
};
use vulkano::shader::ShaderModuleCreateInfo;

use crate::vulkan::VulkanBackend;

pub struct GraphicsShader {
    pub vertex: Arc<ShaderModule>,
    pub fragment: Arc<ShaderModule>,
}

impl GraphicsShader {
    pub fn new(renderer: VulkanBackend, vs: &str, fs: &str) -> Result<GraphicsShader> {
        unsafe {
            let frag_bin = Self::compile_shader(vs, naga::ShaderStage::Vertex)?;
            let vert_bin = Self::compile_shader(fs, naga::ShaderStage::Fragment)?;

            let vertex = ShaderModule::new(
                renderer.device.clone(),
                ShaderModuleCreateInfo::new(&vert_bin),
            )?;
            let fragment = ShaderModule::new(
                renderer.device.clone(),
                ShaderModuleCreateInfo::new(&frag_bin),
            )?;

            Ok(GraphicsShader { vertex, fragment })
        }
    }

    pub fn compile_shader(source: &str, stage: naga::ShaderStage) -> anyhow::Result<Vec<u32>> {
        let mut frontend = Frontend::default();
        let options = Options::from(stage);
        let module = frontend.parse(&options, source)?;

        let info = Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module)?;

        let spv_options = naga::back::spv::Options {
            lang_version: (1, 0),
            flags: spv::WriterFlags::empty(),
            ..Default::default()
        };

        let mut spv_source = Vec::<u32>::new();
        spv::Writer::new(&spv_options)?.write(
            &module,
            &info,
            Some(&PipelineOptions {
                shader_stage: stage,
                entry_point: "main".into(),
            }),
            &None,
            &mut spv_source,
        )?;

        Ok(spv_source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_shader_vertex() {
        let glsl = r#"
            #version 450
            layout(location = 0) in vec3 position;
            
            void main() {
                gl_Position = vec4(position, 1.0);
            }
            "#;

        let spv = GraphicsShader::compile_shader(glsl, naga::ShaderStage::Vertex)
            .expect("Failed to compile vertex shader");

        println!("{spv:?}");
        assert!(!spv.is_empty(), "SPIR-V output should not be empty");
    }
}
