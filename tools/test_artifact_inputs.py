"""Discovery tests, including Cargo compiler observations (private host fixtures)."""

import copy
import os
import re
import shutil
import subprocess

import pytest
from artifact_inputs import discover, explain, unchanged

CONTEXT = {
    "target": {"triple": "x86_64-unknown-linux-gnu", "bytes": None},
    "effective_flags": [],
    "effective_config": {},
    "toolchain": {"rustc": "fixture compiler", "sysroot": "fixture sysroot"},
    "environment": {"SOURCE_DATE_EPOCH": "1"},
    "execution_policy": "fixture-only",
}


def write(root, name, text):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def command(root, *arguments):
    env = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("CARGO_")
        and key not in {"RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTDOCFLAGS"}
    }
    completed = subprocess.run(
        ["cargo", *arguments], cwd=root, capture_output=True, text=True, env=env, check=False
    )
    assert completed.returncode == 0, completed.stderr
    return completed


@pytest.fixture
def fixture(tmp_path):
    root = tmp_path / "checkout"
    write(
        root,
        "Cargo.toml",
        '[workspace]\nresolver="2"\nmembers=["body","dep","unrelated","optional","devonly"]\n[profile.release]\nopt-level=1\n[profile.release.package.unrelated]\nopt-level=2\n',
    )
    for name in ("body", "dep", "unrelated", "optional", "devonly"):
        text = f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n'
        if name == "body":
            text += (
                '[dependencies]\ndep={path="../dep",default-features=false,features=["base"]}\n'
                'optional={path="../optional",optional=true}\n[dev-dependencies]\n'
                'devonly={path="../devonly"}\n[features]\ndefault=["enabled"]\n'
                'enabled=["dep/extra"]\noptional-feature=["dep:optional","optional/extra"]\n'
                'weak=["optional?/extra"]\n'
            )
        elif name == "unrelated":
            text += '[dependencies]\ndep={path="../dep",features=["unrelated"]}\n'
        elif name in {"dep", "optional"}:
            text += (
                '[features]\ndefault=["default-extra"]\nbase=[]\nextra=[]\n'
                "default-extra=[]\nunrelated=[]\n"
            )
        write(root, f"{name}/Cargo.toml", text)
        write(root, f"{name}/src/lib.rs", "pub fn value() -> u32 { 1 }\n")
    command(root, "generate-lockfile", "--offline")
    return root


def project(root, **kwargs):
    result = discover(root, "body", CONTEXT, workspace=root, **kwargs)
    assert result["eligible"], result["reasons"]
    return result


def test_unrelated_member_lock_profile_and_formatting_preserve_key(fixture):
    baseline = project(fixture)
    manifest = fixture / "Cargo.toml"
    manifest.write_text(
        manifest.read_text().replace('"devonly"]', '"devonly","added"]')
        + "\n[profile.release.package.added]\nopt-level=3\n"
    )
    write(fixture, "added/Cargo.toml", '[package]\nname="added"\nversion="1.0.0"\nedition="2021"\n')
    write(fixture, "added/src/lib.rs", "pub fn added() {}\n")
    command(fixture, "generate-lockfile", "--offline")
    write(fixture, "unrelated/src/lib.rs", "pub fn unrelated_changed() {}\n")
    body = fixture / "body/Cargo.toml"
    body.write_text("# unrelated formatting\n" + body.read_text())
    after = project(fixture)
    assert after["action_key"] == baseline["action_key"]
    assert after["whole_manifest_key"] != baseline["whole_manifest_key"]
    assert (
        after["provenance"]["reads"]["Cargo.toml"] != baseline["provenance"]["reads"]["Cargo.toml"]
    )


def test_changed_source_added_deleted_and_same_filename_content(fixture):
    baseline = project(fixture)
    write(fixture, "dep/new-untracked.dat", "new data")
    added = project(fixture)
    assert added["action_key"] != baseline["action_key"]
    (fixture / "dep/new-untracked.dat").unlink()
    assert project(fixture)["action_key"] == baseline["action_key"]
    write(fixture, "dep/src/lib.rs", "pub fn value() -> u32 { 2 }\n")
    assert project(fixture)["action_key"] != baseline["action_key"]


def test_relevant_dependency_features_and_profile(fixture):
    baseline = project(fixture)
    manifest = fixture / "body/Cargo.toml"
    original = manifest.read_text()
    manifest.write_text(original.replace('features=["base"]', 'features=["base","extra"]'))
    # Default already enables extra, so this semantically equivalent request hits.
    assert project(fixture)["action_key"] == baseline["action_key"]
    assert project(fixture, default_features=False)["action_key"] != project(fixture)["action_key"]
    manifest.write_text(original.replace('features=["base"]', 'features=["base","default-extra"]'))
    assert project(fixture)["action_key"] != baseline["action_key"]
    manifest.write_text(original)
    with (fixture / "Cargo.toml").open("a") as output:
        output.write("\n[profile.release.package.dep]\nopt-level=3\n")
    assert project(fixture)["action_key"] != baseline["action_key"]


def test_relocated_checkout_normalizes_paths_and_reads_no_outputs(fixture, tmp_path):
    baseline = project(fixture)
    other = tmp_path / "other location"
    shutil.copytree(fixture, other)
    write(other, "dep/target/poison", "private output must not be read")
    after = project(other)
    assert baseline["action_key"] == after["action_key"]
    assert not any("target/" in path for path in after["provenance"]["reads"])


def test_conditional_optional_build_script_eligibility(fixture):
    write(fixture, "optional/build.rs", "fn main() {}")
    assert project(fixture)["eligible"]
    weak = discover(fixture, "body", CONTEXT, features=("weak",), workspace=fixture)
    assert not weak["eligible"] and "weak optional" in weak["reasons"][0]
    active = discover(fixture, "body", CONTEXT, features=("optional-feature",), workspace=fixture)
    assert not active["eligible"]
    assert "build script unsupported" in active["reasons"][0]


@pytest.mark.parametrize("construct", ["target", "workspace-dep", "proc-macro", "unknown", "patch"])
def test_unsupported_constructs_are_explicit(fixture, construct):
    manifest = fixture / "body/Cargo.toml"
    if construct == "target":
        manifest.write_text(
            manifest.read_text() + "\n[target.'cfg(unix)'.dependencies]\ndep={path=\"../dep\"}\n"
        )
    elif construct == "workspace-dep":
        manifest.write_text(
            manifest.read_text().replace(
                'dep={path="../dep",default-features=false,features=["base"]}',
                "dep={workspace=true}",
            )
        )
        with (fixture / "Cargo.toml").open("a") as output:
            output.write('\n[workspace.dependencies]\ndep={path="dep"}\n')
    elif construct == "proc-macro":
        manifest.write_text(manifest.read_text() + "\n[lib]\nproc-macro=true\n")
    elif construct == "unknown":
        manifest.write_text(manifest.read_text() + "\n[unknown-policy]\nunsafe-to-ignore=true\n")
    else:
        with (fixture / "Cargo.toml").open("a") as output:
            output.write('\n[patch.crates-io]\nsome-package={path="dep"}\n')
    result = discover(fixture, "body", CONTEXT, workspace=fixture)
    assert not result["eligible"]
    assert result["reasons"]
    assert result["action_key"] is None


def test_external_symlink_and_manifest_observation_rejected(fixture, tmp_path):
    external = tmp_path / "secret"
    external.write_text("outside")
    (fixture / "body/external").symlink_to(external)
    result = discover(fixture, "body", CONTEXT, workspace=fixture)
    assert not result["eligible"] and "symlink" in result["reasons"][0]
    (fixture / "body/external").unlink()
    write(fixture, "body/src/lib.rs", 'pub const MANIFEST: &str = include_str!("../Cargo.toml");')
    result = discover(fixture, "body", CONTEXT, workspace=fixture)
    assert not result["eligible"] and "include" in result["reasons"][0]


def test_environment_target_flags_toolchain_and_missing_context(fixture):
    baseline = project(fixture)
    for field in CONTEXT:
        changed = copy.deepcopy(CONTEXT)
        changed[field] = {"changed": True}
        result = discover(fixture, "body", changed, workspace=fixture)
        assert result["action_key"] != baseline["action_key"]
    absent = discover(fixture, "body", {}, workspace=fixture)
    assert absent["eligible"] and absent["source_projection_key"]
    assert absent["action_key"] is None
    assert explain(baseline["descriptor"], absent["descriptor"])
    write(fixture, "body/src/lib.rs", 'pub const VALUE: &str = env!("UNDECLARED");')
    assert not discover(fixture, "body", CONTEXT, workspace=fixture)["eligible"]


@pytest.mark.parametrize(
    "features,default_features", [((), True), ((), False), (("optional-feature",), False)]
)
def test_projection_matches_real_pinned_cargo_compiler_commands(
    fixture, tmp_path, features, default_features
):
    result = project(fixture, features=features, default_features=default_features)
    args = [
        "build",
        "--lib",
        "--offline",
        "--locked",
        "-p",
        "body",
        "-vv",
        "--target-dir",
        str(tmp_path / "private-target"),
    ]
    if features:
        args.extend(["--features", ",".join(features)])
    if not default_features:
        args.append("--no-default-features")
    compiled = command(fixture, *args)
    observed = {}
    for line in compiled.stderr.splitlines():
        if "Running `" not in line or "--crate-name " not in line:
            continue
        matched = re.search(r"--crate-name ([a-z_]+)", line)
        assert matched is not None
        name = matched.group(1)
        observed[name] = sorted(re.findall(r'--cfg \'feature="([^"]+)"\'', line))
    expected = {
        entry["package"]["name"].replace("-", "_"): entry["features"]
        for entry in result["descriptor"]["packages"].values()
    }
    assert observed == expected, {
        "compiler": observed,
        "projection": expected,
        "commands": compiled.stderr,
    }
    assert "unrelated" not in observed and "devonly" not in observed
    assert "unrelated" not in observed["dep"]
    # The private metadata resolution leaves every original fixture byte intact.
    assert (fixture / "Cargo.lock").is_file()


def test_weak_optional_metadata_negative_control(fixture, tmp_path):
    """Cargo metadata does not prove exact features: preserve this counterexample."""
    projected = discover(
        fixture, "body", CONTEXT, features=("weak",), default_features=False, workspace=fixture
    )
    assert not projected["eligible"] and "weak optional" in projected["reasons"][0]
    built = command(
        fixture,
        "build",
        "--lib",
        "--offline",
        "--locked",
        "-p",
        "body",
        "--no-default-features",
        "--features",
        "weak",
        "-vv",
        "--target-dir",
        str(tmp_path / "private-target"),
    )
    assert "--crate-name optional" not in built.stderr
    assert "--crate-name body" in built.stderr


@pytest.mark.parametrize(
    "source",
    [
        'pub const X: &str = include_str /* comment */ ! { "../../../external" };',
        '#[cfg_attr(unix, path="../../../external")] mod hidden;',
        'pub const X: &str = env /* nested /* comment */ comment */ ! { "UNDECLARED" };',
        'pub const X: &str = include_str!["../Cargo.toml"];',
    ],
)
def test_source_audit_handles_macro_delimiters_comments_and_conditional_paths(fixture, source):
    write(fixture, "body/src/lib.rs", source)
    result = discover(fixture, "body", CONTEXT, workspace=fixture)
    assert not result["eligible"] and result["reasons"]


def test_character_quote_cannot_hide_external_include(fixture):
    write(
        fixture,
        "body/src/lib.rs",
        "pub const QUOTE: char = '\"';\n"
        + 'pub const X: &str = include_str!("../../../external");',
    )
    result = discover(fixture, "body", CONTEXT, workspace=fixture)
    assert not result["eligible"] and "include" in result["reasons"][0]


def test_forced_staticlib_is_a_distinct_action(fixture):
    normal = project(fixture)
    forced = project(fixture, force_staticlib=True)
    assert normal["action_key"] != forced["action_key"]
    assert (
        forced["descriptor"]["request"]["invocation"] == "cargo rustc --lib --crate-type staticlib"
    )


def test_a_dependency_s_lib_name_is_the_edge_name(fixture):
    # scsidisk-rust's `[lib] name = "scsidisk"`: the extern name Cargo gives the edge.
    # Discovery's private resolution manifest once dropped it, predicted `dep_lib`,
    # and the worker's real graph refused the result (nvme-rust, 2026-10-16).
    manifest = fixture / "dep/Cargo.toml"
    manifest.write_text(manifest.read_text() + '[lib]\nname = "renamed"\n')
    edges = project(fixture)["descriptor"]["packages"]["body"]["edges"]
    assert {"name": "renamed", "package": "dep"} in edges


def test_package_lints_are_keyed_and_inherited_lints_refused(fixture):
    manifest = fixture / "body/Cargo.toml"
    base = manifest.read_text()
    manifest.write_text(base + '[lints.rust]\nunexpected_cfgs = { level = "warn" }\n')
    first = project(fixture)
    manifest.write_text(base + '[lints.rust]\nunexpected_cfgs = { level = "deny" }\n')
    assert project(fixture)["action_key"] != first["action_key"]
    manifest.write_text(base + "[lints]\nworkspace = true\n")
    result = discover(fixture, "body", CONTEXT, workspace=fixture)
    assert not result["eligible"]
    assert any("lints" in reason for reason in result["reasons"])


def test_a_manifest_dir_data_include_outside_the_package_is_keyed(fixture):
    # hal-rust embeds hal/halx86/legacy/bus/pci_*.ids: data from outside its root,
    # which must change the key when it changes and never be read from outputs.
    write(fixture, "data/table.ids", "v1\n")
    write(
        fixture,
        "body/src/lib.rs",
        "pub static T: &[u8] = include_bytes!(concat!(\n"
        '    env!("CARGO_MANIFEST_DIR"),\n    "/../data/table.ids"\n));\n',
    )
    first = project(fixture)
    assert first["descriptor"]["data_inputs"] == {
        "data/table.ids": {
            "sha256": first["descriptor"]["data_inputs"]["data/table.ids"]["sha256"],
            "bytes": 3,
        }
    }
    assert "data/table.ids" in first["provenance"]["reads"]
    write(fixture, "data/table.ids", "v2\n")
    assert project(fixture)["action_key"] != first["action_key"]
    # Into a build output: refused.
    write(fixture, "build-amd64/gen.ids", "x\n")
    write(
        fixture,
        "body/src/lib.rs",
        "pub static T: &[u8] = include_bytes!(concat!(\n"
        '    env!("CARGO_MANIFEST_DIR"), "/../build-amd64/gen.ids"));\n',
    )
    result = discover(fixture, "body", CONTEXT, workspace=fixture)
    assert not result["eligible"] and "data include" in result["reasons"][0]
    # include! of code from outside the package is not a data include.
    write(fixture, "data/code.rs", "pub fn f() {}\n")
    write(
        fixture,
        "body/src/lib.rs",
        'include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/code.rs"));\n',
    )
    assert not discover(fixture, "body", CONTEXT, workspace=fixture)["eligible"]


def test_bodies_without_a_data_include_keep_their_key_shape(fixture):
    assert "data_inputs" not in project(fixture)["descriptor"]


def test_unchanged_is_true_until_any_input_moves(fixture):
    """Request revalidation's fast path: equal stamps mean equal inputs; any edit, added
    or removed file, or touch of a manifest the discovery read makes it re-discover."""
    baseline = project(fixture)
    assert unchanged(baseline)
    for edit in (
        lambda: write(fixture, "dep/src/lib.rs", "pub fn value() -> u32 { 2 }\n"),
        lambda: write(fixture, "dep/src/new.rs", "// added\n"),
        lambda: (fixture / "body/src/lib.rs").unlink(),
        lambda: os.utime(fixture / "Cargo.lock"),
    ):
        current = project(fixture) if (fixture / "body/src/lib.rs").exists() else None
        if current is None:
            write(fixture, "body/src/lib.rs", "pub fn value() -> u32 { 1 }\n")
            current = project(fixture)
        assert unchanged(current)
        edit()
        assert not unchanged(current)


def test_unchanged_never_vouches_for_an_unstamped_or_ineligible_discovery():
    assert not unchanged({"eligible": True, "provenance": {}})
    assert not unchanged({"eligible": False, "provenance": {"stamps": {"/": [0]}}})


def install_native_artifact_tool(root):
    """Install the shipping native executable in a disposable helper distribution."""
    from pathlib import Path

    repo = Path(__file__).resolve().parents[1]
    subprocess.run([str(repo / "tools/build-infra"), "--help"], check=True, capture_output=True)
    tools = root / "tools"
    tools.mkdir(exist_ok=True)
    shutil.copy2(repo / "tools/build_infra_compat.py", tools / "build_infra_compat.py")
    shutil.copy2(repo / ".cache/vibeos-build/vibeos-build", tools / "build-infra")


def test_native_canonical_bytes_match_python_for_unicode_large_integers_and_floats():
    import hashlib
    import json
    import random

    from artifact_inputs import canonical, digest

    rng = random.Random(419)
    numbers = [0.0, -0.0, 1e-7, 1e-6, 1e-5, 1e-4, 1e15, 1e16, 1e20, 5e-324, 1.7976931348623157e308]
    numbers += [rng.uniform(-1, 1) * 10.0 ** rng.randint(-300, 300) for _ in range(100)]
    value = {"unicode": "café\n雪😀", "integers": [-(2**200), 2**200], "floats": numbers}
    expected = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()
    assert canonical(value) == expected
    assert digest(value) == hashlib.sha256(expected).hexdigest()


def test_native_transport_reuses_a_server_and_restarts_for_source_or_environment_changes(
    tmp_path, monkeypatch
):
    import concurrent.futures
    import json

    from tools import build_infra_compat as bridge

    install_native_artifact_tool(tmp_path)
    bridge._close_sessions()
    monkeypatch.setattr(bridge, "REPO", tmp_path)
    sources = tmp_path / "tools/vibeos-build"
    (sources / "src").mkdir(parents=True)
    (sources / "Cargo.toml").write_text("fixture source fingerprint")
    (sources / "Cargo.lock").write_text("fixture lock fingerprint")
    (tmp_path / "rust-toolchain.toml").write_text("fixture pin fingerprint")
    policy = sources / "src/policy.rs"
    policy.write_text("first")

    def request(index):
        return bridge.call(
            "artifact_inputs_canonical", {"value": {"index": index}}, root=tmp_path
        ).encode("ascii")

    try:
        assert request(0) == b'{"index":0}'
        server = bridge._sessions[str(tmp_path)].process.pid
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as workers:
            results = list(workers.map(request, range(24)))
        assert [json.loads(row) for row in results] == [{"index": i} for i in range(24)]
        assert bridge._sessions[str(tmp_path)].process.pid == server
        child = os.fork()
        if child == 0:
            try:
                assert request(24) == b'{"index":24}'
                bridge._close_sessions()
                os._exit(0)
            except BaseException:
                os._exit(1)
        _, status = os.waitpid(child, 0)
        assert os.waitstatus_to_exitcode(status) == 0
        assert bridge._sessions[str(tmp_path)].process.poll() is None
        with pytest.raises(bridge.NativeError):
            bridge.call("artifact_inputs__toml", {"value": None}, root=tmp_path)
        assert request(25) == b'{"index":25}'
        policy.write_text("second")
        assert request(26) == b'{"index":26}'
        replacement = bridge._sessions[str(tmp_path)].process.pid
        assert replacement != server
        monkeypatch.setenv("NATIVE_ARTIFACT_ENV_FRESHNESS", "changed")
        assert request(27) == b'{"index":27}'
        assert bridge._sessions[str(tmp_path)].process.pid != replacement
    finally:
        bridge._close_sessions()


def test_native_transport_refuses_source_changes_during_a_request(fixture, tmp_path, monkeypatch):
    import concurrent.futures
    import time
    from pathlib import Path

    from tools import build_infra_compat as bridge

    installation = tmp_path / "native-distribution"
    installation.mkdir()
    install_native_artifact_tool(installation)
    bridge._close_sessions()
    monkeypatch.setattr(bridge, "REPO", installation)
    started, release = tmp_path / "started", tmp_path / "release"
    cargo = tmp_path / "cargo-controlled"
    actual = Path.home() / ".rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu/bin/cargo"
    import shlex

    cargo.write_text(
        "#!/bin/sh\n"
        f"touch {shlex.quote(str(started))}\n"
        f"while [ ! -e {shlex.quote(str(release))} ]; do sleep 0.01; done\n"
        f'exec {shlex.quote(str(actual))} "$@"\n'
    )
    cargo.chmod(0o755)
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
            pending = executor.submit(
                bridge.call,
                "artifact_inputs_discover",
                {"repo_root": fixture, "body": "body", "workspace": fixture, "cargo": str(cargo)},
                root=installation,
            )
            deadline = time.monotonic() + 20
            while not started.exists() and not pending.done() and time.monotonic() < deadline:
                time.sleep(0.01)
            # Replacing the installed binary atomically changes its identity without
            # modifying the running executable's mapped inode.
            binary = installation / "tools/build-infra"
            replacement = binary.with_name("replacement")
            shutil.copy2(binary, replacement)
            os.replace(replacement, binary)
            release.touch()
            assert started.exists(), "native operation never reached its declared Cargo producer"
            with pytest.raises(bridge.NativeError, match="sources changed during artifact request"):
                pending.result(timeout=20)
        assert bridge.call(
            "artifact_inputs_discover",
            {"repo_root": fixture, "body": "body", "workspace": fixture, "cargo": str(cargo)},
        )["eligible"]
    finally:
        release.touch()
        bridge._close_sessions()


def test_native_transport_cancellation_cannot_leak_a_response_into_the_next_request(
    fixture, tmp_path
):
    import shlex
    import signal
    import threading
    import time
    from pathlib import Path

    from tools import build_infra_compat as bridge

    bridge._close_sessions()
    started, release = tmp_path / "started", tmp_path / "release"
    cargo = tmp_path / "cargo-controlled"
    actual = Path.home() / ".rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu/bin/cargo"
    cargo.write_text(
        "#!/bin/sh\n"
        f"touch {shlex.quote(str(started))}\n"
        f"while [ ! -e {shlex.quote(str(release))} ]; do sleep 0.01; done\n"
        f'exec {shlex.quote(str(actual))} "$@"\n'
    )
    cargo.chmod(0o755)

    def interrupt():
        deadline = time.monotonic() + 20
        while not started.exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        if started.exists():
            os.kill(os.getpid(), signal.SIGINT)
        time.sleep(0.1)
        release.touch()

    controller = threading.Thread(target=interrupt)
    controller.start()
    try:
        with pytest.raises(KeyboardInterrupt):
            bridge.call(
                "artifact_inputs_discover",
                {"repo_root": fixture, "body": "body", "workspace": fixture, "cargo": str(cargo)},
            )
        assert not bridge._sessions
        assert (
            bridge.call("artifact_inputs_canonical", {"value": "next response"})
            == '"next response"'
        )
    finally:
        release.touch()
        controller.join(timeout=20)
        bridge._close_sessions()


@pytest.mark.parametrize(
    "payload", ["malformed", '{"unrelated":1}', '{"error":null}', '{"ok":1,"error":{}}']
)
def test_native_transport_discards_malformed_response_streams(tmp_path, monkeypatch, payload):
    import sys

    from tools import build_infra_compat as bridge

    bridge._close_sessions()
    (tmp_path / "tools").mkdir()
    executable = tmp_path / "tools/build-infra"
    executable.write_text(
        f"#!{sys.executable}\nimport sys\n"
        f"for line in sys.stdin:\n    print({payload!r}, flush=True)\n"
    )
    executable.chmod(0o755)
    monkeypatch.setattr(bridge, "REPO", tmp_path)
    try:
        with pytest.raises(bridge.NativeError, match="response"):
            bridge.call(
                "artifact_inputs_canonical", {"value": "must not be accepted"}, root=tmp_path
            )
        assert not bridge._sessions
    finally:
        bridge._close_sessions()


@pytest.mark.parametrize("resource_owned", [False, True])
def test_native_transport_preserves_bounded_permits_without_leaking_them_to_metadata(
    tmp_path, resource_owned
):
    import fcntl

    from tools import build_infra_compat as bridge

    bridge._close_sessions()
    path = tmp_path / "inherited-admission.lock"
    permit = path.open("w+")
    fcntl.flock(permit, fcntl.LOCK_EX)
    os.set_inheritable(permit.fileno(), True)
    connection = None
    try:
        if resource_owned:
            connection = bridge.DedicatedSession("--api-jsonl")
            assert connection.call("artifact_inputs_canonical", {"value": 1}) == "1"
        else:
            assert bridge.call("artifact_inputs_canonical", {"value": 1}) == "1"
        permit.close()
        with path.open("r+") as contender:
            if resource_owned:
                with pytest.raises(BlockingIOError):
                    fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)
                connection.close()
            fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)
    finally:
        permit.close()
        if connection:
            connection.close()
        bridge._close_sessions()
