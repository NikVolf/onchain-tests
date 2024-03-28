use anyhow::Context;
use gear_test_runtime::ControlSignal;
use gtest::{Program, System};
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

mod control_bus;

pub fn run_from_bin_path(bin_path_file: impl AsRef<Path>) -> anyhow::Result<()> {
    let wasm_base = std::fs::read_to_string(bin_path_file.as_ref().to_path_buf())
        .with_context(|| format!("Reading {:?}", bin_path_file.as_ref().to_path_buf()))?;

    let mut bin_base = bin_path_file.as_ref().to_path_buf();
    bin_base.pop();

    let wasm_bin_path = bin_base.join(PathBuf::from(format!("{wasm_base}.opt.wasm")));

    let test_bin_path = bin_base.join(PathBuf::from(format!("{wasm_base}_test.opt.wasm")));

    run_tests(wasm_bin_path, test_bin_path)
}

pub fn run_from_dir(directory: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut path = directory.as_ref().to_path_buf();
    path.push(".binpath");

    run_from_bin_path(path)
}

pub fn run_tests(
    program_wasm_path: impl AsRef<Path>,
    progrm_test_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let system = System::new();
    system.init_logger();

    // test_program
    let test_program = Program::from_file(&system, progrm_test_path);
    let res = test_program.send_bytes(0, vec![]); // empty initialization for test program
    assert!(!res.main_failed());

    // code under test (code_hash)
    let code_hash = system.submit_code(program_wasm_path);

    // control bus program (for results telemetry)
    let control_bus = control_bus::ControlBus::default();
    let running_state = control_bus.running_state();
    let control = Program::mock(&system, control_bus);
    // apparently it also should be initialized
    let res = control.send_bytes(0, vec![]);
    assert!(!res.main_failed());

    // actual test run
    let res = test_program.send(
        0,
        ControlSignal::Test {
            code_hash: code_hash.into_bytes().into(),
            control_bus: control.id().into_bytes().into(),
        },
    );
    assert!(!res.main_failed());

    let report = running_state.read().unwrap().report();
    println!("\n{}", report);

    if !report.success() {
        anyhow::bail!("Some test failed or unfinished!");
    }

    Ok(())
}

fn generate_cargo_args() -> Vec<String> {
    ["build".to_string()]
        .into_iter()
        .chain(std::env::args().skip(2))
        .collect()
}

fn main() -> anyhow::Result<()> {
    let builder_artifacts_file = NamedTempFile::new()?;
    let builder_artifacts_path = builder_artifacts_file.path().as_os_str();

    let mut cargo_args = generate_cargo_args();
    cargo_args.push("--config".to_string());
    cargo_args.push(format!(
        "env.GEAR_BUILDER_ARTIFACTS=\"{}\"",
        builder_artifacts_path.to_string_lossy()
    ));

    eprintln!("Running cargo {}", cargo_args.clone().join(" "));

    let build_out = Command::new("cargo")
        .args(cargo_args)
        .stderr(Stdio::inherit())
        .output()?;

    if !build_out.status.success() {
        anyhow::bail!(
            "Cargo command failed (cargo {})",
            generate_cargo_args().join(" ")
        );
    }

    for line in BufReader::new(builder_artifacts_file).lines() {
        let line = line?;
        let clone_line = line.clone();
        let paths = clone_line.split("|").collect::<Vec<_>>();
        if paths.len() != 2 {
            anyhow::bail!("Got this from artifacts dump: '{}'. This is invalid, should be '<wasm_path>|<wasm_test_path>'", line);
        }

        run_tests(paths[0], paths[1])?;
    }

    // file for gear_test_builder artifacts report;

    Ok(())
}
