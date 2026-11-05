"""Exercise shipping nested-Cargo dependency propagation through real Ninja builds."""

import os
import subprocess
import sys
import time
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("watch_nested", [True, False])
@pytest.mark.parametrize("crate_types", ['["staticlib"]', '["staticlib", "rlib"]'])
def test_nested_inputs_rebuild_consumers_only(tmp_path, crate_types, watch_nested):
    root = tmp_path / "with spaces"
    root.mkdir()
    compiled = False
    initial_mtime_ns = time.time_ns() - 3_600_000_000_000

    def write(path, text):
        p = root / path
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
        # Exercise dependency propagation with a definite edit ordering. Host
        # timestamps can put a just-written input behind a prior Cargo output
        # (the sibling BuildFixture already handles this). Do not sleep/retry
        # or alter dependency edges: the executable must still change correctly.
        stamp = initial_mtime_ns
        if compiled:
            stamp = max(time.time_ns(), max(f.stat().st_mtime_ns for f in root.rglob("*")))
            stamp += 1_000_000
        os.utime(p, ns=(stamp, stamp))

    write(
        "Cargo.toml",
        '[workspace]\nmembers = ["outer", "body", "shared", "unrelated"]\nresolver = "2"\n',
    )
    for name in ("outer", "body", "shared", "unrelated"):
        extra = f"[lib]\ncrate-type = {crate_types}\n" if name == "body" else ""
        if name == "body":
            extra += '[dependencies]\nshared = { path = "../shared" }\n'
        write(
            f"{name}/Cargo.toml",
            f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n{extra}',
        )
    write("shared/src/lib.rs", "pub fn value() -> u32 { 1 }")
    write("unrelated/src/lib.rs", "pub fn unrelated() {}")
    write("body/asset.txt", "2")
    write("body/schema.txt", "3")
    write(
        "body/build.rs",
        """fn main() {
        println!("cargo:rerun-if-changed=schema.txt");
        println!("cargo:rustc-env=SCHEMA={}", std::fs::read_to_string("schema.txt").unwrap());
    }""",
    )
    write(
        "body/src/lib.rs",
        """#[no_mangle] pub extern "C" fn value() -> u32 {
        shared::value() + include_str!("../asset.txt").trim().parse::<u32>().unwrap()
            + env!("SCHEMA").trim().parse::<u32>().unwrap()
    }""",
    )
    write(
        "outer/src/main.rs",
        'extern "C" { fn value() -> u32; } fn main() { println!("{}", unsafe { value() }); }',
    )
    watch = 'cargo_deps::watch(&lib.with_extension("d"), &lib, &root);' if watch_nested else ""
    write(
        "outer/build.rs",
        f'''#[path = "{REPO}/sdk/lib/rust/reactos-link/src/cargo_deps.rs"] mod cargo_deps;
    #[path = "{REPO}/sdk/lib/rust/reactos-link/src/compiler_env.rs"] mod compiler_env;
    #[path = "{REPO}/sdk/lib/rust/reactos-link/src/body_storage.rs"] mod body_storage;
    use std::{{env, fs, path::PathBuf, process::Command}};
    fn main() {{
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let root = manifest.parent().unwrap().to_path_buf();
        let storage =
            body_storage::Shared::acquire(&root, "host", "body", b"host-test", 1).unwrap();
        let target = storage.target_dir();
        let mut command = Command::new("cargo");
        command.args(["build", "--offline", "-p", "body", "--target-dir"])
            .arg(&target).current_dir(&root);
        compiler_env::scrub_outer_compiler(&mut command);
        assert!(command.status().unwrap().success());
        let lib = target.join("debug/libbody.a");
        {watch}
        println!("cargo:rerun-if-changed=build.rs");
        let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        let owned = storage.snapshot(&lib, &out, b"body-default").unwrap();
        drop(storage);
        println!("cargo:rustc-link-search=native={{}}", owned.parent().unwrap().display());
        println!("cargo:rustc-link-lib=static=body");
        let count = root.join("invocations");
        let old = fs::read_to_string(&count).unwrap_or_default();
        fs::write(count, old + "run\\n").unwrap();
    }}''',
    )
    write(
        "CMakeLists.txt",
        f'''cmake_minimum_required(VERSION 3.20)
project(nested_deps NONE)
cmake_policy(SET CMP0116 NEW)
set(binary "${{CMAKE_BINARY_DIR}}/cargo/debug/outer")
add_custom_command(OUTPUT "${{CMAKE_BINARY_DIR}}/app"
COMMAND cargo build --offline -p outer --target-dir "${{CMAKE_BINARY_DIR}}/cargo"
COMMAND "${{CMAKE_COMMAND}}" -E copy "${{binary}}" "${{CMAKE_BINARY_DIR}}/app"
COMMAND "{REPO}/tools/build-infra" retarget-depfile
  --source "${{binary}}.d" --cargo-output "${{binary}}"
  --target "${{CMAKE_BINARY_DIR}}/app" --output "${{CMAKE_BINARY_DIR}}/app.d"
DEPFILE "${{CMAKE_BINARY_DIR}}/app.d"
DEPENDS "${{CMAKE_SOURCE_DIR}}/outer/build.rs"
WORKING_DIRECTORY "${{CMAKE_SOURCE_DIR}}" VERBATIM)
add_custom_target(app_target ALL DEPENDS "${{CMAKE_BINARY_DIR}}/app")
''',
    )
    # The same cache wrapper supplied to the outer build must reach the nested
    # compiler after the shipping toolchain/flags scrub.
    write(
        "cache-wrapper.py",
        f"#!{sys.executable}\n"
        + """import os
import subprocess
import sys
with open(os.environ["WRAPPER_LOG"], "a") as log:
    log.write(" ".join(sys.argv[1:]) + "\\n")
sys.exit(subprocess.call(sys.argv[1:]))
""",
    )
    wrapper = root / "cache-wrapper.py"
    wrapper.chmod(0o755)
    env = dict(
        os.environ,
        RUSTC_WRAPPER=str(wrapper),
        CARGO_BUILD_RUSTC_WRAPPER="",
        WRAPPER_LOG=str(root / "wrapper.log"),
    )
    for key in ("CARGO_BUILD_TARGET", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        env.pop(key, None)

    def run(*args):
        cp = subprocess.run(args, cwd=root, env=env, capture_output=True, text=True, timeout=90)
        assert cp.returncode == 0, cp.stdout + cp.stderr
        return cp.stdout

    run("cmake", "-S", ".", "-B", "build", "-G", "Ninja")

    def build(expected):
        run("cmake", "--build", "build")
        assert run(str(root / "build/app")).strip() == str(expected)

    build(6)
    compiled = True
    assert "--crate-name shared" in (root / "wrapper.log").read_text()
    count = (root / "invocations").read_text()
    write("unrelated/src/lib.rs", "pub fn unrelated() -> u32 { 999 }")
    build(6)
    assert (root / "invocations").read_text() == count
    write("shared/src/lib.rs", "pub fn value() -> u32 { 10 }")
    if not watch_nested:
        # Negative control: newer input timestamps cannot repair missing edges.
        build(6)
        assert (root / "invocations").read_text() == count
        return
    build(15)
    write("body/asset.txt", "20")
    build(33)
    write("body/schema.txt", "30")
    build(60)
    # A manifest-only edit must reach nested Cargo too.
    p = root / "shared/Cargo.toml"
    write("shared/Cargo.toml", p.read_text() + '\n[features]\ndefault = ["extra"]\nextra = []\n')
    write(
        "shared/src/lib.rs",
        '#[cfg(feature="extra")] pub fn value() -> u32 { 100 }\n'
        '#[cfg(not(feature="extra"))] pub fn value() -> u32 { 10 }',
    )
    build(150)
    write("shared/Cargo.toml", p.read_text().replace('default = ["extra"]', "default = []"))
    build(60)


def test_cmake_boolean_changes_rebuild_only_readers(tmp_path):
    """Media-only reconfiguration must not relink Cargo cells; real flags must."""
    root = tmp_path / "config inputs with spaces"
    root.mkdir()
    source = REPO / "sdk/lib/rust/reactos-link/src/cmake_config.rs"
    context = REPO / "sdk/cmake/cargo_link_context.cmake"
    options = {"ccache": "ENABLE_CCACHE", "winkd": "_WINKD_", "gdb": "GDB", "database": None}
    (root / "Cargo.toml").write_text(
        '[workspace]\nmembers=["ccache", "winkd", "gdb", "database"]\nresolver="2"\n'
    )
    (root / "record.cmake").write_text('file(APPEND "${LOG}" "run\\n")\n')
    commands = []
    for package, option in options.items():
        crate = root / package
        (crate / "src").mkdir(parents=True)
        (crate / "Cargo.toml").write_text(
            f'[package]\nname="{package}"\nversion="0.1.0"\nedition="2021"\n'
        )
        (crate / "src/main.rs").write_text('fn main() { println!("{}", env!("VALUE")); }')
        read_value = (
            f'let value = cmake_config::read(&build, "{option}").unwrap().unwrap();\n'
            if option is not None
            else 'let db = build.join("compile_commands.json");\n'
            'println!("cargo:rerun-if-changed={}", db.display());\n'
            "let data = std::fs::read_to_string(db).unwrap();\n"
            'let value = data.contains("-DFIXTURE_COMMAND_FLAG=1");\n'
        )
        (crate / "build.rs").write_text(
            f'#[path="{source}"] mod cmake_config;\n'
            "fn main() {\n"
            'let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());\n'
            'let build = root.parent().unwrap().join("build");\n'
            + read_value
            + 'println!("cargo:rustc-env=VALUE={}", value);\n'
            'let count = root.join("invocations");\n'
            "let old = std::fs::read_to_string(&count).unwrap_or_default();\n"
            'std::fs::write(count, old + "run\\n").unwrap();\n'
            "}\n"
        )
        commands.append(
            f'''set(binary "${{CMAKE_BINARY_DIR}}/{package}-cargo/debug/{package}")
add_custom_command(OUTPUT "${{CMAKE_BINARY_DIR}}/{package}"
 COMMAND "${{CMAKE_COMMAND}}" "-DLOG=${{CMAKE_SOURCE_DIR}}/{package}/edges"
   -P "${{CMAKE_SOURCE_DIR}}/record.cmake"
 COMMAND cargo build --offline -p {package} --target-dir "${{CMAKE_BINARY_DIR}}/{package}-cargo"
 COMMAND "${{CMAKE_COMMAND}}" -E copy "${{binary}}" "${{CMAKE_BINARY_DIR}}/{package}"
 COMMAND "{REPO}/tools/build-infra" retarget-depfile
   --source "${{binary}}.d" --cargo-output "${{binary}}"
   --target "${{CMAKE_BINARY_DIR}}/{package}" --output "${{CMAKE_BINARY_DIR}}/{package}.d"
   --dependency-source "${{CMAKE_BINARY_DIR}}/compile_commands.json"
   --dependency-snapshot "${{CMAKE_BINARY_DIR}}/reactos-link-compile-commands.json"
 DEPFILE "${{CMAKE_BINARY_DIR}}/{package}.d"
 DEPENDS "${{CMAKE_SOURCE_DIR}}/{package}/build.rs" "{source}"
 WORKING_DIRECTORY "${{CMAKE_SOURCE_DIR}}" VERBATIM)
add_custom_target({package}_target ALL DEPENDS "${{CMAKE_BINARY_DIR}}/{package}")
add_dependencies({package}_target reactos_cargo_compile_db)
'''
        )
    (root / "CMakeLists.txt").write_text(
        "cmake_minimum_required(VERSION 3.20)\nproject(config_inputs C)\n"
        "set(CMAKE_EXPORT_COMPILE_COMMANDS ON)\n"
        "cmake_policy(SET CMP0116 NEW)\n"
        'option(ENABLE_CCACHE "cache" OFF)\noption(_WINKD_ "kd" OFF)\noption(GDB "gdb" OFF)\n'
        'set(MEDIA_ONLY OFF CACHE BOOL "unrelated media staging")\n'
        'option(COMMAND_FLAG "real compile option" OFF)\n'
        "add_library(sample OBJECT sample.c)\n"
        "target_compile_definitions(sample PRIVATE FIXTURE_COMMAND_FLAG=$<BOOL:${COMMAND_FLAG}>)\n"
        f'include("{context}")\n'
        'reactos_cargo_link_context("${CMAKE_SOURCE_DIR}" fixture amd64 ignored)\n'
        + "".join(commands)
    )
    (root / "sample.c").write_text("int fixture;\n")
    env = dict(os.environ, RUSTC_WRAPPER="", CARGO_BUILD_RUSTC_WRAPPER="", CARGO_INCREMENTAL="0")
    for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_TARGET", "RUSTUP_TOOLCHAIN"):
        env.pop(key, None)

    def run(*argv):
        result = subprocess.run(argv, cwd=root, env=env, capture_output=True, text=True, timeout=90)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout

    def configure(*args):
        run("cmake", "-S", ".", "-B", "build", "-G", "Ninja", *args)

    def build(expected_counts, expected_values):
        run("ninja", "-C", "build", "-j", "1")
        assert {
            name: len((root / name / "invocations").read_text().splitlines()) for name in options
        } == dict(zip(options, expected_counts, strict=True))
        assert {
            name: len((root / name / "edges").read_text().splitlines()) for name in options
        } == dict(zip(options, expected_counts, strict=True))
        assert [run(str(root / "build" / name)).strip() for name in options] == expected_values

    configure()
    build([1, 1, 1, 1], ["false", "false", "false", "false"])
    configure("-DMEDIA_ONLY=ON")
    build([1, 1, 1, 1], ["false", "false", "false", "false"])
    configure("-DENABLE_CCACHE=ON")
    build([2, 1, 1, 1], ["true", "false", "false", "false"])
    configure("-D_WINKD_=ON")
    build([2, 2, 1, 1], ["true", "true", "false", "false"])
    configure("-DGDB=ON")
    build([2, 2, 2, 1], ["true", "true", "true", "false"])
    configure("-DENABLE_CCACHE=OFF", "-D_WINKD_=OFF", "-DGDB=OFF")
    build([3, 3, 3, 1], ["false", "false", "false", "false"])
    # Identical configuration leaves both projection and Cargo outputs fresh.
    configure()
    build([3, 3, 3, 1], ["false", "false", "false", "false"])
    configure("-DCOMMAND_FLAG=ON")
    build([3, 3, 3, 2], ["false", "false", "false", "true"])


def test_cmake_boolean_projection_refusals(tmp_path):
    binary = tmp_path / "cmake-config-tests"
    source = REPO / "sdk/lib/rust/reactos-link/src/cmake_config.rs"
    for command in (
        ["rustc", "--edition=2021", "--test", str(source), "-o", str(binary)],
        [str(binary)],
    ):
        result = subprocess.run(command, cwd=REPO, capture_output=True, text=True, timeout=90)
        assert result.returncode == 0, result.stdout + result.stderr
