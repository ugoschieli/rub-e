fn main() {
    // Compile the engine's shared shader modules (e.g. `camera`) into the `etib`
    // WESL package, embedded into the crate via `wesl_pkg!(etib)` and made
    // available to dependent shaders through `import etib::...`.
    wesl::PkgBuilder::new("etib")
        .scan_root("src/shaders/lib.wesl")
        .expect("failed to scan WESL shader files")
        .validate()
        .inspect_err(|e| eprintln!("{e}"))
        .expect("invalid WESL shader package")
        .build_artifact()
        .expect("failed to build WESL package artifact");
}
