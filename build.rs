use std::{env, fs, path::PathBuf};

const SHADER_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/shaders");
const SHADER_EXTENSIONS: &[&str] = &["vert", "frag"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // NOTE(GetAGripGal): For build.rs this has to be read at runtime since in build.rs `target_os` defaults to compiler host not target host.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS")?;

    if target_os == "macos" {
        let spirv_binaries = compile_into_spirv(&out_dir)?;
        compile_into_msl(&spirv_binaries)?;
    } else {
        compile_into_spirv(&out_dir)?;
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shaders");
    Ok(())
}

/// Setup the spirv compiler.
fn setup_spirv_compiler<'a>() -> Result<(shaderc::Compiler, shaderc::CompileOptions<'a>), Box<dyn std::error::Error>> {
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    options.set_include_callback(|name, include_type, _parent, _depth| match include_type {
        shaderc::IncludeType::Relative => {
            let out = fs::read_to_string(name).map_err(|e| e.to_string())?;

            Ok(shaderc::ResolvedInclude {
                resolved_name: name.to_string(),
                content: out,
            })
        }
        shaderc::IncludeType::Standard => {
            let path = PathBuf::from(SHADER_DIR).join(name);
            let out = fs::read_to_string(path).map_err(|e| e.to_string())?;

            Ok(shaderc::ResolvedInclude {
                resolved_name: name.to_string(),
                content: out,
            })
        }
    });

    Ok((compiler, options))
}

/// Compile the shaders into spirv.
/// Returns the paths to the resulting spirv files.
fn compile_into_spirv(out_dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let (compiler, options) = setup_spirv_compiler()?;

    let mut paths = Vec::new();
    for shader in fs::read_dir(SHADER_DIR).unwrap() {
        let path = shader.unwrap().path();

        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            if !SHADER_EXTENSIONS.contains(&ext) {
                continue;
            }
        } else {
            continue;
        }

        let name = path.file_name().unwrap().to_str().unwrap().to_string() + ".spv";
        let dest_path = out_dir.join(name);

        println!("{}", path.display());
        let bin = compiler
            .compile_into_spirv(
                &fs::read_to_string(&path).unwrap(),
                shaderc::ShaderKind::InferFromSource,
                &path.display().to_string(),
                "main",
                Some(&options),
            )
            .unwrap();

        fs::write(&dest_path, bin.as_binary_u8()).unwrap();

        paths.push(dest_path);
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    }

    Ok(paths)
}

/// Compile the spir-v shaders into msl.
fn compile_into_msl(spriv_binaries: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    use spirv_cross2::{Module, targets::Msl};

    /// Read the spir-v files as words.
    /// NOTE(GetAGripGal): We have to do this becuase spirv_cross2 expects the shaders in this format.
    fn read_spirv_words(path: &PathBuf) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        let bytes = fs::read(path)?;

        if bytes.len() % 4 != 0 {
            return Err("SPIR-V file length isn't a multiple of 4 bytes".into());
        }

        let words = bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        Ok(words)
    }

    for shader_path in spriv_binaries.iter() {
        use spirv_cross2::compile::CompilableTarget;

        let shader = read_spirv_words(shader_path)?;

        let module = Module::from_words(&shader);
        let compiler = spirv_cross2::Compiler::<Msl>::new(module)?;

        let options = Msl::options();
        let artifact = compiler.compile(&options)?;

        let msl_path = shader_path.with_extension("metal");
        std::fs::write(msl_path, artifact.to_string())?;
    }
    Ok(())
}
