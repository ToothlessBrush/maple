fn main() {
    //link to glfw lib in ./lib/glfw3.lib
    println!("cargo:rustc-link-search=native=lib");
    println!("cargo:rustc-link-lib=dylib=glfw3");
}
