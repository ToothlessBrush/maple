use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn main() {
    #[cfg(target_os = "windows")]
    {
        //link to glfw lib in ./lib/glfw3.lib
        println!("cargo:rustc-link-search=native=lib");
        println!("cargo:rustc-link-lib=dylib=glfw3");
    }
    // gl_generator to generate the gl bindings

    let dest = env::var("OUT_DIR").unwrap();
    let file = File::create(Path::new(&dest).join("bindings.rs")).unwrap();
    let mut writer = BufWriter::new(file);

    Registry::new(
        Api::Gl,
        (4, 6),
        Profile::Core,
        Fallbacks::All,
        ["GL_ARB_sparse_texture", "GL_EXT_direct_state_access"],
    )
    .write_bindings(GlobalGenerator, &mut writer)
    .unwrap();
}
