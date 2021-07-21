fn main() {
    let lhapdf = pkg_config::Config::new()
        .atleast_version("6")
        .probe("lhapdf")
        .unwrap();

    let mut build = cxx_build::bridge("src/lib.rs");

    for include_path in lhapdf.include_paths {
        build.include(include_path);
    }

    build.compile("lhapdf-rust-cxx-bridge");

    for lib_path in lhapdf.link_paths {
        println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());
    }

    for lib in lhapdf.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
}
