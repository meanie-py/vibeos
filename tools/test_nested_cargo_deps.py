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
COMMAND "{sys.executable}" "{REPO}/tools/retarget_cargo_depfile.py"
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
