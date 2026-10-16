"""Discovery tests, including Cargo compiler observations (private host fixtures)."""

import copy
import os
import re
import shutil
import subprocess

import pytest
from artifact_inputs import discover, explain

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
