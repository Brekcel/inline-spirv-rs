#[allow(unused_imports)]
use crate::{
    CompilationFeedback,
    InputSourceLanguage,
    OptimizationLevel,
    ShaderCompilationConfig,
    ShaderKind,
    TargetEnvironmentType,
    TargetSpirvVersion,
};

impl From<crate::ShaderKind> for naga::ShaderStage {
    fn from(value: crate::ShaderKind) -> Self {
        use naga::ShaderStage;
        match value {
            ShaderKind::Unknown => ShaderStage::Vertex,
            ShaderKind::Vertex => ShaderStage::Vertex,
            ShaderKind::Fragment => ShaderStage::Fragment,
            ShaderKind::Compute => ShaderStage::Compute,
            ShaderKind::Mesh => ShaderStage::Mesh,
            ShaderKind::Task => ShaderStage::Task,

            ShaderKind::RayGeneration => unimplemented!(),
            ShaderKind::Intersection => unimplemented!(),
            ShaderKind::AnyHit => unimplemented!(),
            ShaderKind::ClosestHit => unimplemented!(),
            ShaderKind::Miss => unimplemented!(),
            ShaderKind::Callable => unimplemented!(),
            ShaderKind::TesselationControl => unimplemented!(),
            ShaderKind::TesselationEvaluation => unimplemented!(),
            ShaderKind::Geometry => unimplemented!(),
        }
    }
}

pub(crate) fn compile(
    src: &str,
    _path: Option<&str>,
    cfg: &ShaderCompilationConfig,
) -> Result<CompilationFeedback, String> {
    use naga::{
        back::spv::WriterFlags,
        valid::{Capabilities, ValidationFlags, Validator},
    };

    let shader_stage: naga::ShaderStage = cfg.kind.into();

    let module = match cfg.lang {
        #[cfg(feature = "wgsl_naga")]
        InputSourceLanguage::Wgsl | InputSourceLanguage::Unknown => {
            naga::front::wgsl::parse_str(src).map_err(|e| e.emit_to_string(src))
        }
        #[cfg(feature = "glsl_naga")]
        #[allow(unreachable_patterns)]
        InputSourceLanguage::Glsl | InputSourceLanguage::Unknown => {
            let mut front_end = naga::front::glsl::Frontend::default();
            let options = naga::front::glsl::Options::from(shader_stage);
            front_end
                .parse(&options, &src)
                .map_err(|e| e.emit_to_string(src))
        }
        _ => return Err("unsupported source language".to_owned()),
    }?;
    let mut opts = naga::back::spv::Options::default();
    match (cfg.env_ty, cfg.spv_ver) {
        (TargetEnvironmentType::Vulkan, TargetSpirvVersion::Spirv1_0) => {
            opts.lang_version = (1, 0);
        }
        (TargetEnvironmentType::Vulkan, TargetSpirvVersion::Spirv1_3) => {
            opts.lang_version = (1, 3);
        }
        (TargetEnvironmentType::Vulkan, TargetSpirvVersion::Spirv1_5) => {
            opts.lang_version = (1, 5);
        }
        (TargetEnvironmentType::Vulkan, TargetSpirvVersion::Spirv1_6) => {
            opts.lang_version = (1, 6);
        }
        (TargetEnvironmentType::OpenGL, TargetSpirvVersion::Spirv1_0) => {
            opts.lang_version = (1, 0);
        }
        (TargetEnvironmentType::WebGpu, TargetSpirvVersion::Spirv1_0) => {
            opts.lang_version = (1, 0);
        }
        _ => return Err("unsupported target".to_owned()),
    };
    if cfg.debug {
        opts.flags.insert(WriterFlags::DEBUG);
    } else {
        opts.flags.remove(WriterFlags::DEBUG);
    }
    if cfg.y_flip {
        opts.flags.insert(WriterFlags::ADJUST_COORDINATE_SPACE);
    } else {
        opts.flags.remove(WriterFlags::ADJUST_COORDINATE_SPACE);
    }

    let pipeline_opts = naga::back::spv::PipelineOptions {
        shader_stage,
        entry_point: cfg.entry.clone(),
    };

    // Attempt to validate WGSL, error if invalid
    let info = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .map_err(|e| format!("{:?}", e))?;
    let spv = naga::back::spv::write_vec(&module, &info, &opts, Some(&pipeline_opts))
        .map_err(|e| format!("{:?}", e))?;
    let feedback = CompilationFeedback {
        spv,
        dep_paths: Vec::new(),
    };
    Ok(feedback)
}
