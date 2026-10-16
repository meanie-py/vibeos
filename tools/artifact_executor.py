"""Isolated, pinned body executor for the builtin Windows targets. No store or hits.

Run only through host-jobs and an externally held disk reservation. Trusted
provisioned toolchain/runtime mounts are hashed; they must not be modified by
their host owner during a run. Read-only mounts protect against worker writes,
not host-side writes. No claim of a hostile-code security boundary is made.
"""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import os
import re
import shutil
import signal
import socket
import stat
import subprocess
import sys
import time
from pathlib import Path

from tools import artifact_inputs, artifact_worker
from tools.artifact_inputs import _toml, canonical, discover, tomllib
from tools.artifact_worker import capture

PIN = "1.92.0-x86_64-unknown-linux-gnu"
CANON = Path("/tmp/vibeos")
WORKSPACE = Path("sdk/lib/rust")
IMPLEMENTATION_REPO = Path(artifact_inputs.__file__).resolve().parents[1]
PREPARED_SCHEMA = "vibeos-prepared-body-inputs-v1"
# The builtin triple each architecture's eligible bodies build on. Custom target JSONs
# (the i386 kernel's i686-kernel) need build-std and are not an audited recipe.
TARGETS = {
    "amd64": "x86_64-pc-windows-gnu",
    "i386": "i686-pc-windows-gnu",
    "arm64": "aarch64-pc-windows-gnullvm",
}
AMD64 = TARGETS["amd64"]
# The exact RUSTFLAGS reactos-link gives an eligible nested body build, by (arch,
# kernel). Kernel-mode bodies on a builtin triple restore frame pointers; every i386
# crate builds without SSE. The worker compiles with exactly these, so they are part
# of the action identity; anything else is refused rather than approximated.
FLAGS = {
    ("amd64", False): "",
    ("amd64", True): "-C force-frame-pointers=yes",
    ("arm64", False): "",
    ("arm64", True): "-C force-frame-pointers=yes",
    ("i386", False): "-C target-feature=-sse,-sse2",
    ("i386", True): "-C target-feature=-sse,-sse2 -C force-frame-pointers=yes",
}


def file_digest(path: Path) -> dict:
    before = path.stat()
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    after = path.stat()
    if (before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (
        after.st_ino,
        after.st_size,
        after.st_mtime_ns,
        after.st_ctime_ns,
    ):
        raise RuntimeError(f"input changed while hashing: {path}")
    return {"sha256": digest.hexdigest(), "bytes": after.st_size}


def allocated(root: Path) -> dict:
    seen = set()
    logical = physical = files = 0
    for directory, _, names in os.walk(root):
        for name in names:
            try:
                stat = (Path(directory) / name).lstat()
            except FileNotFoundError:
                continue  # compiler temporary removed between directory/stat
            logical += stat.st_size
            files += 1
            identity = (stat.st_dev, stat.st_ino)
            if identity not in seen:
                physical += stat.st_blocks * 512
                seen.add(identity)
    return {
        "logical_bytes": logical,
        "allocated_bytes": physical,
        "unique_inodes": len(seen),
        "files": files,
    }


def value_digest(value: dict) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def implementation_identity() -> dict:
    """Hash the implementation that interprets and projects compiler inputs."""
    return {
        "executor_source": file_digest(Path(__file__)),
        "support_sources": {
            "tools/artifact_inputs.py": file_digest(Path(artifact_inputs.__file__)),
            "tools/artifact_worker.py": file_digest(Path(artifact_worker.__file__)),
        },
    }


def tree_identity(root: Path) -> dict:
    """Inventory every visible compiler input, including directory membership.

    No output exclusions apply to a prepared snapshot/projection. Unlike live
    output accounting, a missing or non-regular entry is an integrity failure.
    """
    if root.is_symlink() or not root.is_dir():
        raise RuntimeError(f"prepared input root is not a real directory: {root}")
    files = {}
    directories = []
    for current, names, entries in os.walk(root, followlinks=False):
        names.sort()
        entries.sort()
        directory = Path(current)
        directories.append(directory.relative_to(root).as_posix())
        for name in names + entries:
            path = directory / name
            mode = path.lstat().st_mode
            if stat.S_ISLNK(mode) or not (stat.S_ISREG(mode) or stat.S_ISDIR(mode)):
                raise RuntimeError(f"prepared input is a symlink or non-regular entry: {path}")
            if stat.S_ISREG(mode):
                files[path.relative_to(root).as_posix()] = file_digest(path)
    inventory = {"directories": sorted(directories), "files": files}
    return {"sha256": value_digest(inventory), "files": len(files)}


def _provisioned_copy(tool: dict) -> dict:
    """Validate and detach a supervisor's already-hashed trusted tool recipe.

    This does not rehash its sysroot. Run still verifies the actual complete
    toolchain after compilation before accepting an output.
    """
    result = json.loads(canonical(tool))
    if not result.get("rustc", "").startswith("rustc 1.92.0 "):
        raise RuntimeError("provisioned recipe requires pinned rustc 1.92.0")
    inputs = result.get("mounted_inputs")
    if not isinstance(inputs, dict):
        raise RuntimeError("provisioned recipe is missing mounted input identities")
    expected_tools = {
        "/native-tools/bin/llvm-dlltool",
        "/native-tools/lib/libLLVM.so.19.1",
    }
    if set(result.get("native_tools", {})) != expected_tools:
        raise RuntimeError("provisioned native-tool mapping differs from the audited recipe")
    required = {"/toolchain/bin/cargo", "/toolchain/bin/rustc", *expected_tools}
    libraries = result.get("runtime_libraries")
    if not isinstance(libraries, list):
        raise RuntimeError("provisioned recipe is missing runtime library mapping")
    for path in libraries:
        if not isinstance(path, str) or not Path(path).is_absolute() or ".." in Path(path).parts:
            raise RuntimeError("provisioned runtime library path is not canonical")
        required.add(path)
    for path in result["native_tools"].values():
        if not isinstance(path, str) or not Path(path).is_absolute():
            raise RuntimeError("provisioned native executable path is not absolute")
    for path in required:
        entry = inputs.get(path, {})
        if not isinstance(entry, dict) or not re.fullmatch("[0-9a-f]{64}", entry.get("sha256", "")):
            raise RuntimeError(f"provisioned recipe is missing a byte digest: {path}")
        if not isinstance(entry.get("bytes"), int) or entry["bytes"] < 0:
            raise RuntimeError(f"provisioned recipe is missing a byte size: {path}")
    return result


def provision(toolchain: Path) -> dict:
    """Exact ELF runtime allowlist plus full read-only compiler/sysroot tree."""
    inputs = {}
    libraries = set()
    # GNU dlltool uses rustc's random --temp-prefix in COFF symbol names,
    # despite deterministic archive timestamps. LLVM writes the imports directly
    # and ignores temporary prefixes. Keep its executable basename for multicall
    # dispatch and provision its $ORIGIN/../lib runtime at the canonical location.
    llvm = (IMPLEMENTATION_REPO / ".build-tools/llvm-mingw").resolve()
    native_tools = {
        "/native-tools/bin/llvm-dlltool": str((llvm / "bin/llvm-dlltool").resolve()),
        "/native-tools/lib/libLLVM.so.19.1": str((llvm / "lib/libLLVM.so.19.1").resolve()),
    }
    native_sources = {Path(source).resolve() for source in native_tools.values()}
    for source in native_sources:
        if not source.is_file():
            raise RuntimeError(f"required deterministic import-library input missing: {source}")
    for target, source in native_tools.items():
        inputs[target] = file_digest(Path(source))
    for executable in (
        toolchain / "bin/cargo",
        toolchain / "bin/rustc",
        *(Path(path) for path in native_tools.values()),
    ):
        result = subprocess.run(
            ["ldd", str(executable)], capture_output=True, text=True, check=True
        )
        for line in result.stdout.splitlines():
            match = re.search(r"(?:=>\s*)?(/\S+)", line)
            if match:
                path = Path(match.group(1)).absolute()
                if (
                    not path.resolve().is_relative_to(toolchain.resolve())
                    and path.resolve() not in native_sources
                ):
                    libraries.add(path)
    for root, directories, files in os.walk(toolchain):
        directories.sort()
        for name in sorted(files):
            path = Path(root) / name
            if path.is_symlink():
                if not path.resolve().is_relative_to(toolchain.resolve()):
                    raise RuntimeError(f"toolchain symlink escapes mounted tree: {path}")
                inputs["/toolchain/" + path.relative_to(toolchain).as_posix()] = {
                    "symlink": os.readlink(path)
                }
            else:
                inputs["/toolchain/" + path.relative_to(toolchain).as_posix()] = file_digest(path)
    for path in sorted(libraries):
        inputs[str(path)] = file_digest(path)
    version = subprocess.run(
        [str(toolchain / "bin/rustc"), "-Vv"], capture_output=True, text=True, check=True
    ).stdout
    if not version.startswith("rustc 1.92.0 "):
        raise RuntimeError("executor requires pinned rustc 1.92.0")
    return {
        "rustc": version,
        "native_tools": native_tools,
        "mounted_inputs": inputs,
        "runtime_libraries": [str(path) for path in sorted(libraries)],
    }


def normalize(snapshot: Path, projection: dict, destination: Path) -> dict:
    """Render the same normal path graph Cargo discovered, with real sources.

    Inactive optional candidates retain manifests and a parse-only source stub;
    their absence from resolved nodes is checked before compilation.
    """
    descriptor = projection["descriptor"]
    active = descriptor["packages"]
    raw_paths = sorted(
        path for path in snapshot.rglob("Cargo.toml") if path != snapshot / WORKSPACE / "Cargo.toml"
    )
    candidates = {
        path.parent.relative_to(snapshot).as_posix(): tomllib.loads(path.read_text())
        for path in raw_paths
    }
    destination.mkdir()
    for key, raw in candidates.items():
        source = snapshot / key
        output = destination / key
        if key in active:
            shutil.copytree(source, output)
            # Capture is read-only. This fresh compiler projection is mutable
            # during setup only, and mounted read-only before the compiler runs.
            for directory, _, files in os.walk(output):
                os.chmod(directory, 0o755)
                for name in files:
                    os.chmod(Path(directory) / name, 0o644)
        else:
            output.mkdir(parents=True)
            (output / "__artifact_inactive.rs").write_text(
                "// inactive Cargo resolution candidate\n"
            )
        manifest = {key: value for key, value in raw.items() if key != "dev-dependencies"}
        if key not in active:
            manifest["lib"] = {"path": "__artifact_inactive.rs"}
        lines = []
        for table, value in sorted(manifest.items()):
            lines.append("[" + table + "]")
            lines.extend(_toml(name) + " = " + _toml(item) for name, item in sorted(value.items()))
        (output / "Cargo.toml").write_text("\n".join(lines) + "\n")
    selected = descriptor["request"]["package"]

    def relative(key):
        return Path(key).relative_to(WORKSPACE).as_posix()

    root_lines = [
        "[workspace]",
        'resolver = "2"',
        "members = " + _toml([relative(selected)]),
        "exclude = " + _toml([relative(key) for key in candidates if key != selected]),
    ]
    root_lines.append("[profile." + descriptor["request"]["profile"] + "]")
    root_lines.extend(
        _toml(key) + " = " + _toml(value)
        for key, value in sorted(descriptor["profiles"].items())
        if value != {}
    )
    (destination / WORKSPACE / "Cargo.toml").write_text("\n".join(root_lines) + "\n")
    lock_lines = ["version = 4"]
    for _key, raw in sorted(candidates.items(), key=lambda item: item[1]["package"]["name"]):
        package = raw["package"]
        lock_lines.extend(
            [
                "[[package]]",
                "name = " + _toml(package["name"]),
                "version = " + _toml(package["version"]),
            ]
        )
        dependencies = sorted(
            dep.get("package", name) for name, dep in raw.get("dependencies", {}).items()
        )
        if dependencies:
            lock_lines.append("dependencies = " + _toml(dependencies))
    (destination / WORKSPACE / "Cargo.lock").write_text("\n".join(lock_lines) + "\n")
    return {
        "active_packages": sorted(active),
        "manifest_candidates": sorted(candidates),
        "files": allocated(destination),
        "workspace_policy": (
            "selected member only; no dev edges; projected release profile; "
            "original relative package roots"
        ),
    }


def _mount(source: str, target: Path, flags: int, filesystem: str | None = None) -> None:
    libc = ctypes.CDLL(None, use_errno=True)
    result = libc.mount(
        source.encode(),
        os.fsencode(target),
        filesystem.encode() if filesystem else None,
        ctypes.c_ulong(flags),
        None,
    )
    if result:
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error), str(target))


def _bind(source: Path, target: Path, readonly: bool) -> None:
    if source.is_dir():
        target.mkdir(parents=True, exist_ok=True)
    else:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.touch()
    _mount(str(source), target, 4096)  # MS_BIND
    if readonly:
        _mount(str(source), target, 4096 | 32 | 1 | 2 | 4)  # remount bind,ro,nosuid,nodev


def inside(config: dict, *, probe: bool = False) -> None:
    root = Path(config["root"])
    # Namespace is already private. Never expose host /proc or arbitrary /lib.
    _mount("none", Path("/"), 16384 | (1 << 18))  # recursive private propagation
    _bind(Path(config["projection"]), root / "tmp/vibeos", True)
    _bind(Path(config["toolchain"]), root / "toolchain", True)
    for target, source in config.get("native_tools", {}).items():
        _bind(Path(source), root / target.lstrip("/"), True)
    for library in config["runtime_libraries"]:
        _bind(Path(library), root / library.lstrip("/"), True)
    for name in ("target", "cargo-home", "home", "scratch"):
        _bind(Path(config[name]), root / name, False)
    (root / "dev").mkdir(exist_ok=True)
    _bind(Path("/dev/null"), root / "dev/null", False)
    (root / "proc").mkdir(exist_ok=True)
    _mount("proc", root / "proc", 2 | 4 | 8, "proc")
    os.chdir(root)
    os.chroot(root)
    os.chdir(CANON / WORKSPACE)
    if probe:
        evidence = {
            "host_checkout_hidden": not Path(config["repo"]).exists(),
            "host_home_hidden": not Path("/home/curi/.cargo").exists(),
            "environment_keys": sorted(config["env"]),
        }
        try:
            (CANON / WORKSPACE / "Cargo.toml").write_text("must fail")
        except OSError as error:
            evidence["readonly_source_errno"] = error.errno
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as connection:
                connection.settimeout(0.2)
                connection.connect(("1.1.1.1", 443))
        except OSError as error:
            evidence["network_connect_errno"] = error.errno
        evidence["network_devices"] = Path("/proc/net/dev").read_text()
        if (
            not evidence["host_checkout_hidden"]
            or evidence.get("readonly_source_errno") != 30
            or "network_connect_errno" not in evidence
        ):
            raise RuntimeError("worker isolation negative control failed")
        print(json.dumps(evidence), flush=True)
        return
    os.execve("/toolchain/bin/cargo", config["argv"], config["env"])


def launch(config_path: Path, probe: bool = False) -> list[str]:
    # Imports execute before chroot. Script-style launch must work without an
    # ambient PYTHONPATH; the compiler still receives only config['env'].
    bootstrap = (
        "import runpy,sys; sys.path.insert(0,sys.argv.pop(1)); "
        "runpy.run_path(sys.argv.pop(1),run_name='__main__')"
    )
    return [
        "unshare",
        "--user",
        "--map-root-user",
        "--mount",
        "--net",
        "--pid",
        "--fork",
        "--kill-child",
        sys.executable,
        "-c",
        bootstrap,
        str(IMPLEMENTATION_REPO),
        str(Path(__file__).resolve()),
        "inside-probe" if probe else "inside",
        str(config_path),
    ]


def validate_graph(output: Path, descriptor: dict, config: dict) -> dict:
    metadata_config = dict(
        config,
        argv=[
            "/toolchain/bin/cargo",
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--offline",
            "--manifest-path",
            "/tmp/vibeos/sdk/lib/rust/Cargo.toml",
        ],
    )
    request = descriptor["request"]
    if request.get("features"):
        metadata_config["argv"].extend(["--features", ",".join(request["features"])])
    if not request.get("default_features", True):
        metadata_config["argv"].append("--no-default-features")
    path = output / "metadata-worker.json"
    path.write_bytes(canonical(metadata_config))
    result = subprocess.run(launch(path), capture_output=True, text=True, check=False)
    if result.returncode:
        raise RuntimeError("isolated graph resolution failed: " + result.stderr)
    metadata = json.loads(result.stdout)
    identities = {
        package["id"]: Path(package["manifest_path"]).parent.relative_to(CANON).as_posix()
        for package in metadata["packages"]
    }
    observed = {
        identities[node["id"]]: {
            "features": sorted(node["features"]),
            "edges": sorted(
                [
                    {"name": edge["name"], "package": identities[edge["pkg"]]}
                    for edge in node["deps"]
                ],
                key=lambda edge: edge["name"],
            ),
        }
        for node in metadata["resolve"]["nodes"]
    }
    expected = {
        key: {"features": package["features"], "edges": package["edges"]}
        for key, package in descriptor["packages"].items()
    }
    if observed != expected:
        raise RuntimeError(
            "normalized execution graph differs from discovered graph: "
            + json.dumps({"expected": expected, "observed": observed}, sort_keys=True)
        )
    return {
        "status": "identical",
        "package_count": len(observed),
        "graph": observed,
        "stderr": result.stderr,
    }


def prepared_identity(config: dict, output: Path, context: dict) -> dict:
    return {
        "schema": PREPARED_SCHEMA,
        "worker_config_sha256": value_digest(config),
        "context_sha256": value_digest(context),
        "tool_recipe_sha256": value_digest(context["toolchain"]),
        "snapshot": tree_identity(output / "snapshot"),
        "projection": tree_identity(output / "projection"),
        "root_scaffold": tree_identity(output / "root"),
    }


def _verify_mount_leaves(config: dict, tool: dict) -> None:
    """Check executables/runtime now; the full sysroot is checked after the miss."""
    paths = {
        "/toolchain/bin/cargo": str(Path(config["toolchain"]) / "bin/cargo"),
        "/toolchain/bin/rustc": str(Path(config["toolchain"]) / "bin/rustc"),
        **tool["native_tools"],
        **{path: path for path in tool["runtime_libraries"]},
    }
    for target, host in paths.items():
        if file_digest(Path(host)) != tool["mounted_inputs"][target]:
            raise RuntimeError(f"provisioned executable/runtime changed: {target}")


def validate_prepared(output: Path, report: dict) -> dict:
    """Reject altered setup, input membership, code or projected compiler bytes."""
    context = report["context"]
    identity = implementation_identity()
    for key, value in identity.items():
        if context["execution_policy"].get(key) != value:
            raise RuntimeError("executor changed since preparation; prepare a fresh worker")
    config = json.loads((output / "worker.json").read_text())
    for name in ("root", "projection", "target", "cargo-home", "home", "scratch"):
        path = Path(config[name])
        expected = output / name
        if path != expected or path.is_symlink() or path.resolve() != expected.absolute():
            raise RuntimeError(f"prepared worker path was redirected: {name}")
    tool = _provisioned_copy(context["toolchain"])
    if (
        config["native_tools"] != tool["native_tools"]
        or config["runtime_libraries"] != tool["runtime_libraries"]
    ):
        raise RuntimeError("prepared mounted tool recipe differs from action context")
    options = config.get("request_options", {})
    expected_context = request_context(
        Path(config["toolchain"]),
        tool,
        report["body"],
        features=tuple(options.get("features", ())),
        default_features=options.get("default_features", True),
        kernel=config.get("kernel", True),
        **config.get("recipe", {}),
    )
    if expected_context != context:
        raise RuntimeError("prepared action context differs from the current exact recipe")
    recorded = report.get("prepared_inputs")
    if not recorded or recorded.get("schema") != PREPARED_SCHEMA:
        raise RuntimeError("prepared worker lacks an audited integrity record; prepare again")
    if prepared_identity(config, output, context) != recorded:
        raise RuntimeError("prepared config, snapshot or normalized projection changed")
    private = {name: tree_identity(output / name) for name in ("cargo-home", "home", "scratch")}
    if private != report.get("initial_private_inputs"):
        raise RuntimeError("prepared private Cargo/home/scratch inputs changed before execution")
    if (
        config["argv"] != context["effective_config"]["argv"]
        or config["env"] != context["environment"]
    ):
        raise RuntimeError("prepared compiler command/environment differs from action context")
    _verify_mount_leaves(config, tool)
    projection = discover(
        output / "snapshot",
        report["body"],
        context,
        cargo=str(Path(config["toolchain"]) / "bin/cargo"),
        **options,
    )
    if (
        not projection["eligible"]
        or projection["source_projection_key"] != report["source_projection_key"]
        or projection["action_key"] != report["action_key"]
    ):
        raise RuntimeError("prepared source/action identity does not match captured bytes")
    return config


def request_context(
    toolchain: Path,
    provisioned: dict,
    body: str,
    *,
    features: tuple[str, ...] = (),
    default_features: bool = True,
    kernel: bool = True,
    target: str = AMD64,
    flags: str | None = None,
) -> dict:
    """Return the exact context used for discovery and a subsequent miss worker.

    `target` is a builtin triple from TARGETS and `flags` the body's exact RUSTFLAGS
    (None: the amd64 recipe's, by `kernel`).

    Provisioning/hash timings and physical worker paths do not affect identity.
    The supervisor may reuse its already-hashed trusted compiler description.
    """
    if not toolchain.is_absolute():
        raise RuntimeError("toolchain root must be absolute")
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*", body):
        raise RuntimeError("body must name one Cargo package")
    if any(not isinstance(feature, str) for feature in features):
        raise RuntimeError("feature names must be strings")
    features = tuple(sorted(set(features)))
    if target not in TARGETS.values():
        raise RuntimeError(f"target is not a builtin recipe triple: {target}")
    if flags is None:
        flags = FLAGS[("amd64", kernel)]
    if flags not in {f for (a, _k), f in FLAGS.items() if TARGETS[a] == target}:
        raise RuntimeError(f"unaudited RUSTFLAGS for {target}: {flags!r}")
    tool = _provisioned_copy(provisioned)
    env = {
        "PATH": "/native-tools/bin:/toolchain/bin",
        "HOME": "/home",
        "CARGO_HOME": "/cargo-home",
        "CARGO_TARGET_DIR": "/target",
        "TMPDIR": "/scratch",
        "RUSTC": "/toolchain/bin/rustc",
        "RUSTFLAGS": (flags + " " if flags else "") + "-C dlltool=/native-tools/bin/llvm-dlltool",
        "CARGO_INCREMENTAL": "0",
        "CARGO_NET_OFFLINE": "true",
        "LC_ALL": "C",
        "TZ": "UTC",
    }
    argv = [
        "/toolchain/bin/cargo",
        "build",
        "--release",
        "--target",
        target,
        "-p",
        body,
        "--manifest-path",
        "/tmp/vibeos/sdk/lib/rust/Cargo.toml",
        "--lib",
        "--locked",
        "--offline",
        "--jobs",
        "1",
        "-vv",
    ]
    if features:
        argv.extend(["--features", ",".join(features)])
    if not default_features:
        argv.append("--no-default-features")
    context = {
        "target": {"builtin": target},
        "effective_flags": env["RUSTFLAGS"],
        "effective_config": {"argv": argv, "config_files": [], "crate_type": "manifest staticlib"},
        "toolchain": tool,
        "environment": env,
        "execution_policy": {
            "canonical_source": "/tmp/vibeos",
            "worker": "user/mount/net/pid namespace + minimal chroot",
            **implementation_identity(),
            "compiler_cache": "disabled",
            "broker": "outer single compile permit; Cargo jobs=1",
            "provisioning": (
                "trusted read-only toolchain/runtime bind mounts; "
                "no host writes permitted during run"
            ),
        },
    }
    return context


def prepare(
    repo: Path,
    body: str,
    toolchain: Path,
    output: Path,
    *,
    features: tuple[str, ...] = (),
    default_features: bool = True,
    kernel: bool = True,
    provisioned: dict | None = None,
    target: str = AMD64,
    flags: str | None = None,
) -> dict:
    started = time.monotonic()
    output.mkdir(parents=True, exist_ok=False)
    tool_started = time.monotonic()
    tool = _provisioned_copy(provisioned if provisioned is not None else provision(toolchain))
    tool_seconds = time.monotonic() - tool_started
    features = tuple(sorted(set(features)))
    context = request_context(
        toolchain,
        tool,
        body,
        features=features,
        default_features=default_features,
        kernel=kernel,
        target=target,
        flags=flags,
    )
    env = context["environment"]
    argv = context["effective_config"]["argv"]
    request_options = {"features": features, "default_features": default_features}
    cargo = str(toolchain / "bin/cargo")
    discovery = discover(repo, body, context, cargo=cargo, **request_options)
    if not discovery["eligible"]:
        raise RuntimeError("ineligible: " + "; ".join(discovery["reasons"]))
    snapshot = output / "snapshot"
    captured = capture(
        repo,
        body,
        context,
        snapshot,
        cargo=cargo,
        expected_projection_key=discovery["source_projection_key"],
        **request_options,
    )
    normalization = normalize(snapshot, discovery, output / "projection")
    for name in ("root", "target", "cargo-home", "home", "scratch"):
        (output / name).mkdir()
    config = {
        "root": str(output / "root"),
        "projection": str(output / "projection"),
        "toolchain": str(toolchain),
        "runtime_libraries": tool["runtime_libraries"],
        "native_tools": tool["native_tools"],
        "repo": str(repo),
        "kernel": kernel,
        "argv": argv,
        "env": env,
        **{name: str(output / name) for name in ("target", "cargo-home", "home", "scratch")},
    }
    config["request_options"] = request_options
    config["recipe"] = {"target": target, "flags": flags}
    (output / "worker.json").write_bytes(canonical(config))
    graph = validate_graph(output, discovery["descriptor"], config)
    # Cargo metadata queries rustc and leaves this cache without compiling.
    # Remove only this known file so the cold experiment starts truly empty.
    for entry in (output / "target").iterdir():
        if entry.name != ".rustc_info.json" or not entry.is_file():
            raise RuntimeError(f"unexpected output from metadata discovery: {entry}")
        entry.unlink()
    report = {
        "schema": "vibeos-isolated-body-measurement-v1",
        "body": body,
        "source_projection_key": discovery["source_projection_key"],
        "action_key": discovery["action_key"],
        "context": context,
        "source_capture": captured,
        "normalization": normalization,
        "graph_validation": graph,
        "toolchain_hash_seconds": tool_seconds,
        "toolchain_hash_mode": "supervisor-provided"
        if provisioned is not None
        else "local full hash",
        "prepare_seconds": time.monotonic() - started,
        "output": str(output),
        "compile_status": "not run",
        "prepared_inputs": prepared_identity(config, output, context),
        "initial_private_inputs": {
            name: tree_identity(output / name) for name in ("cargo-home", "home", "scratch")
        },
    }
    (output / "report.json").write_bytes(canonical(report))
    return report


def _stop_worker(result: subprocess.Popen) -> None:
    """Terminate only the new session owned by this worker, including its init."""
    if result.poll() is not None:
        return
    try:
        os.killpg(result.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        result.wait(timeout=5)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(result.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        result.wait()


def run(output: Path) -> dict:
    """Caller owns a host-jobs slot and disk admission for this entire call."""
    if os.environ.get("VIBEOS_HOST_JOB_HELD") != "1":
        raise RuntimeError("run must be admitted through host-jobs --kind compile")
    output = output.absolute()
    config_path = output / "worker.json"
    report_path = output / "report.json"
    report = json.loads(report_path.read_text())
    if report.get("compile_status") != "not run" or any((output / "target").iterdir()):
        raise RuntimeError(
            "cold experiment requires a fresh prepared worker and empty target; "
            "refusing to overwrite evidence"
        )
    with (output / "run.started").open("x") as marker:
        marker.write(str(time.time()))
    report["compile_status"] = "running"
    report_path.write_bytes(canonical(report))
    stage = "prepared-input-validation"
    result = None
    started = None
    try:
        validation_started = time.monotonic()
        config = validate_prepared(output, report)
        report["prepared_input_validation_seconds"] = time.monotonic() - validation_started
        report["prepared_input_validation"] = "identical"
        stage = "isolation-probe"
        probe_started = time.monotonic()
        probe = subprocess.run(
            launch(config_path, True), capture_output=True, text=True, check=True
        )
        report["isolation_probe"] = json.loads(probe.stdout)
        report["isolation_probe_seconds"] = time.monotonic() - probe_started
        stage = "compile-launch"
        started = time.monotonic()
        peak = 0
        with (output / "cargo.log").open("w") as log:
            result = subprocess.Popen(
                launch(config_path), stdout=log, stderr=subprocess.STDOUT, start_new_session=True
            )
            try:
                stage = "compile"
                while result.poll() is None:
                    peak = max(peak, allocated(output / "target")["allocated_bytes"])
                    try:
                        result.wait(timeout=0.2)
                    except subprocess.TimeoutExpired:
                        pass
            finally:
                _stop_worker(result)
                report["compile_seconds"] = time.monotonic() - started
                report["exit_code"] = result.returncode
        archive = (
            output
            / "target"
            / report["context"]["target"]["builtin"]
            / "release"
            / ("lib" + report["body"].replace("-", "_") + ".a")
        )
        if result.returncode != 0 or not archive.is_file():
            raise RuntimeError(f"isolated compilation failed; evidence {report_path}")
        stage = "archive-accounting"
        with archive.open("rb") as stream:
            if stream.read(8) != b"!<arch>\n":
                raise RuntimeError("compiler output is not a static archive")
        report["archive"] = {"path": str(archive), **file_digest(archive)}
        report["retained"] = {
            name: allocated(output / name)
            for name in ("snapshot", "projection", "target", "scratch")
        }
        report["sampled_target_peak_allocated_bytes"] = max(
            peak, report["retained"]["target"]["allocated_bytes"]
        )
        report["peak_method"] = (
            "target recursive allocated blocks sampled every 0.2s; not exact transient high-water"
        )
        report["rustc_invocations"] = sum(
            "Running `" in line and "/toolchain/bin/rustc" in line
            for line in (output / "cargo.log").read_text().splitlines()
        )
        stage = "lane-input-revalidation"
        current = discover(
            Path(config["repo"]),
            report["body"],
            report["context"],
            cargo=str(Path(config["toolchain"]) / "bin/cargo"),
            **config.get("request_options", {}),
        )
        report["lane_input_revalidation"] = (
            "identical"
            if current["eligible"]
            and current["source_projection_key"] == report["source_projection_key"]
            else "changed; result rejected for this lane"
        )
        if report["lane_input_revalidation"] != "identical":
            raise RuntimeError("lane inputs changed; result rejected")
        stage = "prepared-input-postcheck"
        recheck_started = time.monotonic()
        if prepared_identity(config, output, report["context"]) != report["prepared_inputs"]:
            raise RuntimeError("prepared sources changed during execution; result rejected")
        if json.loads(config_path.read_text()) != config:
            raise RuntimeError("prepared worker config changed during execution; result rejected")
        report["prepared_input_postcheck_seconds"] = time.monotonic() - recheck_started
        stage = "toolchain-postcheck"
        verify_started = time.monotonic()
        tool_now = provision(Path(config["toolchain"]))
        report["toolchain_postcheck_seconds"] = time.monotonic() - verify_started
        report["toolchain_postcheck"] = (
            "identical"
            if tool_now == report["context"]["toolchain"]
            else "changed; result rejected"
        )
        if report["toolchain_postcheck"] != "identical":
            raise RuntimeError("toolchain inputs changed; result rejected")
        stage = "implementation-postcheck"
        for key, value in implementation_identity().items():
            if report["context"]["execution_policy"][key] != value:
                raise RuntimeError(
                    "executor implementation changed during execution; result rejected"
                )
        report["compile_status"] = "success"
        report_path.write_bytes(canonical(report))
        return report
    except BaseException as error:
        report["compile_status"] = (
            "interrupted" if isinstance(error, (KeyboardInterrupt, SystemExit)) else "failed"
        )
        report["failure_stage"] = stage
        report["failure"] = {"type": type(error).__name__, "message": str(error)}
        if isinstance(error, subprocess.CalledProcessError):
            report["failure"].update(
                returncode=error.returncode,
                stdout=error.stdout.decode(errors="replace")
                if isinstance(error.stdout, bytes)
                else error.stdout,
                stderr=error.stderr.decode(errors="replace")
                if isinstance(error.stderr, bytes)
                else error.stderr,
            )
        report_path.write_bytes(canonical(report))
        raise


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "run", "inside", "inside-probe"))
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--repo", type=Path, default=Path("/home/curi/Projects/os-playground/vibeos")
    )
    parser.add_argument("--body", choices=("beep-rust", "kernel-rust"), default="beep-rust")
    parser.add_argument("--toolchain", type=Path, default=Path.home() / ".rustup/toolchains" / PIN)
    args = parser.parse_args()
    if args.command.startswith("inside"):
        inside(json.loads(args.output.read_text()), probe=args.command == "inside-probe")
    elif args.command == "prepare":
        report = prepare(
            args.repo.resolve(), args.body, args.toolchain.resolve(), args.output.absolute()
        )
        print(
            json.dumps(
                {key: report[key] for key in ("body", "action_key", "prepare_seconds", "output")}
            )
        )
    else:
        report = run(args.output.absolute())
        print(
            json.dumps(
                {key: report[key] for key in ("body", "action_key", "compile_seconds", "archive")}
            )
        )


if __name__ == "__main__":
    main()
