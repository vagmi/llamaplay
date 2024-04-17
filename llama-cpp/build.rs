use std::env;
use std::path::Path;
use std::path::PathBuf;


// courtesy of https://github.com/rustformers/llm
fn metal_hack(build: &mut cc::Build) {
    const GGML_METAL_METAL_PATH: &str = "llama.cpp/ggml-metal.metal";
    const GGML_METAL_PATH: &str = "llama.cpp/ggml-metal.m";
    const GGML_COMMON_PATH: &str = "llama.cpp/ggml-common.h";

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not defined"));

    let ggml_metal_path = {
        let ggml_metal_metal = std::fs::read_to_string(GGML_METAL_METAL_PATH)
            .expect("Could not read ggml-metal.metal")
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\"', "\\\"");

        let ggml_common = std::fs::read_to_string(GGML_COMMON_PATH).expect("Could not read ggml-common.h")
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\"', "\\\"");

        let includged_ggml_metal_metal = ggml_metal_metal.replace(
            "#include \\\"ggml-common.h\\\"",
            &format!("{ggml_common}")
        );
        print!("{}", &includged_ggml_metal_metal);

        let ggml_metal =
            std::fs::read_to_string(GGML_METAL_PATH).expect("Could not read ggml-metal.m");

        let needle = r#"NSString * src = [NSString stringWithContentsOfFile:path_source encoding:NSUTF8StringEncoding error:&error];"#;
        if !ggml_metal.contains(needle) {
            panic!("ggml-metal.m does not contain the needle to be replaced; the patching logic needs to be reinvestigated. Contact a `llama-cpp-sys-2` developer!");
        }

        // Replace the runtime read of the file with a compile-time string
        let ggml_metal = ggml_metal.replace(
            needle,
            &format!(r#"NSString * src  = @"{includged_ggml_metal_metal}";"#),
        );

        let patched_ggml_metal_path = out_dir.join("ggml-metal.m");
        std::fs::write(&patched_ggml_metal_path, ggml_metal)
            .expect("Could not write temporary patched ggml-metal.m");

        patched_ggml_metal_path
    };

    build.file(ggml_metal_path);
}

fn main() {
    println!("cargo:rerun-if-changed=llama.cpp");

    if !Path::new("llama.cpp/ggml.c").exists() {
        panic!("Run git sumodule update to get llama.cpp")
    }

    let mut ggml = cc::Build::new();
    let mut llama_cpp = cc::Build::new();

    ggml.cpp(false);
    llama_cpp.cpp(true);

        ggml.flag("-mcpu=apple-m1");
        ggml.flag("-pthread");

        llama_cpp.flag("-mcpu=apple-m1");
        llama_cpp.flag("-pthread");


        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
        println!("cargo:rustc-link-lib=framework=MetalKit");

        llama_cpp.define("_DARWIN_C_SOURCE", None);

        // https://github.com/ggerganov/llama.cpp/blob/3c0d25c4756742ebf15ad44700fabc0700c638bd/Makefile#L340-L343
        llama_cpp.define("GGML_USE_METAL", None);
        llama_cpp.define("GGML_USE_ACCELERATE", None);
        llama_cpp.define("ACCELERATE_NEW_LAPACK", None);
        llama_cpp.define("ACCELERATE_LAPACK_ILP64", None);
        println!("cargo:rustc-link-lib=framework=Accelerate");

        metal_hack(&mut ggml);
        ggml.include("./llama.cpp/ggml-metal.h");


    ggml.std("c11")
        .include("./llama.cpp")
        .file("llama.cpp/ggml.c")
        .file("llama.cpp/ggml-alloc.c")
        .file("llama.cpp/ggml-backend.c")
        .file("llama.cpp/ggml-quants.c")
        .define("GGML_USE_K_QUANTS", None);

    llama_cpp
        .define("_XOPEN_SOURCE", Some("600"))
        .include("llama.cpp")
        .std("c++11")
        .file("llama.cpp/llama.cpp")
        .file("llama.cpp/unicode-data.cpp")
        .file("llama.cpp/unicode.cpp");

    // Remove debug log output from `llama.cpp`
    let is_release = env::var("PROFILE").unwrap() == "release";

    if is_release {
        ggml.define("NDEBUG", None);
        llama_cpp.define("NDEBUG", None);
    }


    println!("compiling ggml");
    ggml.compile("ggml");
    println!("compiled ggml");

    println!("compiling llama");
    llama_cpp.compile("llama");
    println!("compiled llama");

    let header = "llama.cpp/llama.h";

    println!("cargo:rerun-if-changed={header}");

    let bindings = bindgen::builder()
        .header(header)
        .derive_partialeq(true)
        .no_debug("llama_grammar_element")
        .prepend_enum_name(false)
        .derive_eq(true)
        .generate()
        .expect("failed to generate bindings for llama.cpp");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings to file");
    let llama_cpp_dir = PathBuf::from("llama.cpp").canonicalize().unwrap();
    println!("cargo:INCLUDE={}", llama_cpp_dir.to_str().unwrap());
    println!("cargo:OUT_DIR={}", out_path.to_str().unwrap());
}
