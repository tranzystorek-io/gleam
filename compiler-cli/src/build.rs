use std::{sync::Arc, time::Instant};

use gleam_core::{
    build::{Built, Codegen, Options, ProjectCompiler},
    manifest::Manifest,
    paths::ProjectPaths,
    Result,
};

use crate::{
    build_lock::BuildLock,
    cli,
    dependencies::UseManifest,
    fs::{self, ConsoleWarningEmitter},
};

pub fn download_dependencies() -> Result<Manifest> {
    let paths = crate::project_paths_at_current_directory();
    crate::dependencies::download(&paths, cli::Reporter::new(), None, UseManifest::Yes)
}

pub fn main(options: Options, manifest: Manifest) -> Result<Built> {
    let paths = crate::project_paths_at_current_directory();
    let perform_codegen = options.codegen;
    let root_config = crate::config::root_config()?;
    let telemetry = Box::new(cli::Reporter::new());
    let io = fs::ProjectIO::new();
    let start = Instant::now();
    let lock = BuildLock::new_target(
        &paths,
        options.mode,
        options.target.unwrap_or(root_config.target),
    )?;
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    tracing::info!("Compiling packages");
    let compiled = {
        let _guard = lock.lock(telemetry.as_ref());
        let compiler = ProjectCompiler::new(
            root_config,
            options,
            manifest.packages,
            telemetry,
            Arc::new(ConsoleWarningEmitter),
            ProjectPaths::new(current_dir),
            io,
        );
        compiler.compile()?
    };

    match perform_codegen {
        Codegen::All | Codegen::DepsOnly => cli::print_compiled(start.elapsed()),
        Codegen::None => cli::print_checked(start.elapsed()),
    };
    Ok(compiled)
}
