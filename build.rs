use std::{env, fs, path::PathBuf};

const SHADER_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/shaders");
const SHADER_EXTENSIONS: &[&str] = &["vert", "frag"];

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // `cfg(target_os)` in a build script is the host, so read the target from the env.
    let target_macos = env::var("CARGO_CFG_TARGET_OS").unwrap() == "macos";
    #[cfg(not(any(feature = "metal", target_os = "macos")))]
    assert!(!target_macos, "cross-compiling for macOS requires the `metal` feature");

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

        #[cfg(any(feature = "metal", target_os = "macos"))]
        if target_macos {
            fs::write(dest_path.with_extension("metal"), compile_into_msl(bin.as_binary())).unwrap();
        }

        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shaders");
}

#[cfg(any(feature = "metal", target_os = "macos"))]
fn compile_into_msl(spirv: &[u32]) -> String {
    use spirv_cross2::{Compiler, Module, compile::CompilableTarget, targets::Msl};

    let compiler = Compiler::<Msl>::new(Module::from_words(spirv)).unwrap();
    compiler.compile(&Msl::options()).unwrap().to_string()
}
