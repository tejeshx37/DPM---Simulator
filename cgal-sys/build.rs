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
];

fn main() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    
    let (gmp_include, gmp_lib) = if cfg!(target_os = "windows") {
        (PathBuf::from("C:\\msys64\\mingw64\\include"), PathBuf::from("C:\\msys64\\mingw64\\lib"))
    } else if cfg!(target_os = "macos") {
        if Path::new("/opt/homebrew/include").exists() {
            (PathBuf::from("/opt/homebrew/include"), PathBuf::from("/opt/homebrew/lib"))
        } else {
            (PathBuf::from("/usr/local/include"), PathBuf::from("/usr/local/lib"))
        }
    } else {
        (PathBuf::from("/usr/include"), PathBuf::from("/usr/lib"))
    };

    let mut build = cxx_build::bridges(["src/lib.rs", "src/triangulation.rs"]);
    build.cpp(true)
        .flag("-std=gnu++17");

    if cfg!(target_os = "windows") {
        build.flag("-U__STRICT_ANSI__")
            .flag("-fext-numeric-literals")
            .flag("-D_UCRT")
            .flag("-D_GNU_SOURCE")
            .flag("-D_WIN32_WINNT=0x0601");
    }

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
