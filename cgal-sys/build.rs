use anyhow::Context;
use std::{env, path::Path, path::PathBuf};

const CPP_FILES: &[&str] = &[
    "cpp/curve.cpp",
    "cpp/num.cpp",
    "cpp/point.cpp",
    "cpp/polygon_set.cpp",
    "cpp/polygon.cpp",
    "cpp/polygon_with_holes.cpp",
    "cpp/triangulation.cpp",
    "cpp/polyhedron_set.cpp",
];

fn main() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    
    let gmp_include = env::var("GMP_INCLUDE_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        if Path::new("/opt/homebrew/include").exists() {
            PathBuf::from("/opt/homebrew/include")
        } else {
            PathBuf::from("/usr/include")
        }
    });

    let gmp_lib = env::var("GMP_LIB_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        if Path::new("/opt/homebrew/lib").exists() {
            PathBuf::from("/opt/homebrew/lib")
        } else {
            PathBuf::from("/usr/lib")
        }
    });


    let mut build = cxx_build::bridges(["src/lib.rs", "src/triangulation.rs"]);
    build.cpp(true)
        .flag("-std=gnu++17");



    build.flag("-w")
        .files(CPP_FILES)
        .includes([
            &manifest_dir.join("include"),
            &boost_sys::headers(),
            &gmp_include,
        ])
        .std("c++17")
        .try_compile("cgal")
        .context("CGAL wrapper compilation failed")?;

    println!("cargo:rustc-link-search=native={}", gmp_lib.display());
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:rustc-link-lib=mpfr");
    
    println!("cargo:rerun-if-changed=cpp");

    Ok(())
}
