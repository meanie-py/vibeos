"""Discovery-only projected inputs for path-only Cargo library builds.

This does not authorize an artifact hit: discovery reads a live checkout, not an
immutable snapshot. The descriptor needs the executor's complete effective context.
Only normal path dependencies and the audited feature grammar below are modeled.
Cargo resolves a private manifest mirror, offline and locked; no checkout writes.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any

try:
    import tomllib  # type: ignore[reportMissingImports]  # Python 3.11+ stdlib
except ImportError:  # Python 3.10 in the repository tooling environment.
    import tomli as tomllib

SCHEMA = "vibeos-body-input-projection-v1-spike"
PACKAGE_KEYS = {
    "name",
    "version",
    "edition",
    "rust-version",
    "authors",
    "description",
    "homepage",
    "repository",
    "license",
    "license-file",
    "readme",
    "keywords",
    "categories",
    "publish",
    "include",
    "exclude",
    "metadata",
    "build",
    "autolib",
    "autobins",
    "autoexamples",
    "autotests",
    "autobenches",
}
TOP_KEYS = {"package", "lib", "dependencies", "dev-dependencies", "features", "lints"}
LIB_KEYS = {
    "name",
    "path",
    "crate-type",
    "required-features",
    "test",
    "doctest",
    "bench",
    "doc",
    "harness",
    "proc-macro",
    "edition",
}
DEP_KEYS = {"path", "version", "package", "features", "default-features", "optional"}
PROFILE_KEYS = {
    "opt-level",
    "debug",
    "split-debuginfo",
    "strip",
    "debug-assertions",
    "overflow-checks",
    "lto",
    "panic",
    "incremental",
    "codegen-units",
    "rpath",
}
CONTEXT_KEYS = {
    "target",
    "effective_flags",
    "effective_config",
    "toolchain",
    "environment",
    "execution_policy",
}
PACKAGE_ENV = {
    "CARGO_PKG_NAME",
    "CARGO_PKG_VERSION",
    "CARGO_PKG_VERSION_MAJOR",
    "CARGO_PKG_VERSION_MINOR",
    "CARGO_PKG_VERSION_PATCH",
    "CARGO_PKG_VERSION_PRE",
    "CARGO_PKG_AUTHORS",
    "CARGO_PKG_DESCRIPTION",
    "CARGO_PKG_HOMEPAGE",
    "CARGO_PKG_REPOSITORY",
    "CARGO_PKG_LICENSE",
    "CARGO_PKG_LICENSE_FILE",
    "CARGO_PKG_RUST_VERSION",
    "CARGO_CRATE_NAME",
    "CARGO_PRIMARY_PACKAGE",
}


class Ineligible(ValueError):
    """An input construct has no audited projection."""


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()


def digest(value: Any) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def _check_keys(value: dict, allowed: set[str], location: str) -> None:
    unknown = sorted(set(value) - allowed)
    if unknown:
        raise Ineligible(f"{location}: unsupported keys {unknown}")


def _under(path: Path, root: Path) -> bool:
    return path == root or root in path.parents


def _toml(value: Any) -> str:
    if isinstance(value, dict):
        return (
            "{ "
            + ", ".join(json.dumps(k) + " = " + _toml(v) for k, v in sorted(value.items()))
            + " }"
        )
    if isinstance(value, list):
        return "[" + ", ".join(_toml(v) for v in value) + "]"
    if isinstance(value, bool):
        return str(value).lower()
    if isinstance(value, (str, int, float)):
        return json.dumps(value)
    raise Ineligible(f"unsupported TOML value {type(value).__name__}")


def _without_comments(text: str) -> str:
    """Preserve strings and positions while removing nested Rust comments."""
    character_pattern = re.compile(r"'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]+\}|.)|[^'\\\n])'")
    raw_pattern = re.compile(r'r(#+)?"')
    output = []
    index = 0
    tokens = re.compile(r"//|/\*|r(?:#+)?\"|[\"']")
    comment_tokens = re.compile(r"/\*|\*/")
    while index < len(text):
        token = tokens.search(text, index)
        if token is None:
            output.append(text[index:])
            break
        output.append(text[index : token.start()])
        index = token.start()
        start = index
        if text.startswith("//", index):
            end = text.find("\n", index)
            index = len(text) if end == -1 else end
            output.append(" " * (index - start))
        elif text.startswith("/*", index):
            depth = 1
            index += 2
            while index < len(text) and depth:
                boundary = comment_tokens.search(text, index)
                if boundary is None:
                    break
                depth += 1 if boundary.group(0) == "/*" else -1
                index = boundary.end()
            if depth:
                raise Ineligible("unterminated Rust block comment")
            output.append(" " * (index - start))
        elif text[index] == "'" and (character := character_pattern.match(text, index)):
            index += len(character.group(0))
            output.append(text[start:index])
        elif text[index] == '"':
            index += 1
            while index < len(text):
                if text[index] == "\\":
                    index += 2
                elif text[index] == '"':
                    index += 1
                    break
                else:
                    index += 1
            output.append(text[start:index])
        elif text[index] == "r" and (raw := raw_pattern.match(text, index)):
            terminal = '"' + (raw.group(1) or "")
            end = text.find(terminal, index + len(raw.group(0)))
            if end == -1:
                raise Ineligible("unterminated Rust raw string")
            index = end + len(terminal)
            output.append(text[start:index])
        else:
            output.append(text[index])
            index += 1
    return "".join(output)


def explain(before: dict, after: dict) -> list[dict]:
    """Return field-level descriptor differences; lists are ordered atomic values."""
    changes: list[dict] = []

    def walk(a: Any, b: Any, path: str) -> None:
        if isinstance(a, dict) and isinstance(b, dict):
            for key in sorted(set(a) | set(b)):
                if key not in a or key not in b:
                    changes.append(
                        {
                            "field": f"{path}/{key}",
                            "before": a.get(key),
                            "after": b.get(key),
                            "absent_before": key not in a,
                            "absent_after": key not in b,
                        }
                    )
                else:
                    walk(a[key], b[key], f"{path}/{key}")
        elif a != b:
            changes.append({"field": path, "before": a, "after": b})

    walk(before, after, "")
    return changes


class Discovery:
    def __init__(self, repo: Path, workspace: Path, context: dict):
        self.repo = repo.resolve()
        self.workspace = workspace.resolve()
        self.context = context
        self.reads: dict[str, dict] = {}
        self.manifests: dict[str, dict] = {}
        self.roots: dict[str, Path] = {}
        self.raw_manifests: dict[str, str] = {}
        self.evidence: dict = {}

    def logical(self, path: Path) -> str:
        if not _under(path.resolve(), self.repo):
            raise Ineligible(f"outside repository root: {path}")
        return path.resolve().relative_to(self.repo).as_posix()

    def read(self, path: Path) -> bytes:
        logical = self.logical(path)
        before = path.stat()
        data = path.read_bytes()
        after = path.stat()
        stamp = (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_mtime_ns,
            before.st_ctime_ns,
        )
        if stamp != (
            after.st_dev,
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
            after.st_ctime_ns,
        ):
            raise Ineligible(f"input changed while reading: {logical}")
        record = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
        if logical in self.reads and self.reads[logical] != record:
            raise Ineligible(f"input changed during discovery: {logical}")
        self.reads[logical] = record
        return data

    def load(self, manifest: Path) -> str:
        manifest = manifest.resolve()
        logical = self.logical(manifest.parent)
        if logical in self.manifests:
            return logical
        data = self.read(manifest)
        raw = tomllib.loads(data.decode())
        _check_keys(raw, TOP_KEYS, self.logical(manifest))
        package = raw.get("package", {})
        _check_keys(package, PACKAGE_KEYS, f"{logical}/package")
        if not isinstance(package.get("name"), str) or not isinstance(package.get("version"), str):
            raise Ineligible(f"{logical}: package name/version must be explicit strings")
        if any(isinstance(v, dict) and v.get("workspace") for v in package.values()):
            raise Ineligible(f"{logical}: inherited workspace package fields unsupported")
        library = raw.get("lib", {})
        _check_keys(library, LIB_KEYS, f"{logical}/lib")
        if raw.get("lints"):
            raise Ineligible(f"{logical}: manifest lints not yet modeled")
        self.manifests[logical] = raw
        self.roots[logical] = manifest.parent
        self.raw_manifests[logical] = data.decode()
        for name, dependency in raw.get("dependencies", {}).items():
            if not isinstance(dependency, dict):
                raise Ineligible(f"{logical}/{name}: only path dependencies supported")
            _check_keys(dependency, DEP_KEYS, f"{logical}/{name}")
            if "path" not in dependency:
                raise Ineligible(f"{logical}/{name}: only explicit path dependencies supported")
            child = self.load(manifest.parent / dependency["path"] / "Cargo.toml")
            expected = dependency.get("package", name)
            if self.manifests[child]["package"]["name"] != expected:
                raise Ineligible(f"{logical}/{name}: dependency package name mismatch")
        return logical

    def metadata(
        self,
        selected: str,
        features: tuple[str, ...],
        default_features: bool,
        cargo: str,
        resolver: str,
    ) -> dict:
        # Stub sources are sufficient for resolution. Every candidate path package
        # appears once; ONLY selected is a workspace member, avoiding metadata's
        # workspace-wide default-feature unification. Dev edges are omitted because
        # the invocation is build --lib, not test/check --all-targets (fixture proven).
        with tempfile.TemporaryDirectory(prefix="vibeos-artifact-resolution-") as temporary:
            private = Path(temporary)
            identities = {key: f"p{index}" for index, key in enumerate(sorted(self.manifests))}
            excluded = [value for key, value in identities.items() if key != selected]
            (private / "Cargo.toml").write_text(
                "[workspace]\nresolver = "
                + _toml(resolver)
                + "\nmembers = "
                + _toml([identities[selected]])
                + "\nexclude = "
                + _toml(excluded)
                + "\n"
            )
            lock_entries = []
            for key, raw in sorted(self.manifests.items()):
                destination = private / identities[key]
                destination.mkdir()
                package = raw["package"]
                lines = [
                    "[package]",
                    "name = " + _toml(package["name"]),
                    "version = " + _toml(package["version"]),
                    "edition = " + _toml(package.get("edition", "2015")),
                    "build = false",
                    "[lib]",
                    'path = "lib.rs"',
                ]
                (destination / "lib.rs").write_text("pub fn resolution_stub() {}\n")
                deps = raw.get("dependencies", {})
                if deps:
                    lines.append("[dependencies]")
                dep_names = []
                for name, dep in sorted(deps.items()):
                    child = self.logical(self.roots[key] / dep["path"])
                    replacement = dict(dep, path="../" + identities[child])
                    lines.append(_toml(name) + " = " + _toml(replacement))
                    dep_names.append(self.manifests[child]["package"]["name"])
                if raw.get("features"):
                    lines.append("[features]")
                    lines.extend(
                        _toml(name) + " = " + _toml(values)
                        for name, values in sorted(raw["features"].items())
                    )
                (destination / "Cargo.toml").write_text("\n".join(lines) + "\n")
                lock_entries.append(
                    {
                        "name": package["name"],
                        "version": package["version"],
                        "dependencies": sorted(dep_names),
                    }
                )
            # Cargo lock package entries are alphabetical, and omit empty deps.
            lock_text = ["version = 4"]
            for entry in sorted(lock_entries, key=lambda item: (item["name"], item["version"])):
                lock_text.extend(
                    [
                        "[[package]]",
                        "name = " + _toml(entry["name"]),
                        "version = " + _toml(entry["version"]),
                    ]
                )
                if entry["dependencies"]:
                    lock_text.append("dependencies = " + _toml(entry["dependencies"]))
            (private / "Cargo.lock").write_text("\n".join(lock_text) + "\n")
            command = [
                cargo,
                "metadata",
                "--format-version",
                "1",
                "--offline",
                "--locked",
                "--manifest-path",
                str(private / identities[selected] / "Cargo.toml"),
            ]
            if features:
                command.extend(["--features", ",".join(features)])
            if not default_features:
                command.append("--no-default-features")
            environment = {
                key: value
                for key, value in os.environ.items()
                if key in {"PATH", "HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "LD_LIBRARY_PATH"}
            }
            # Empty private CARGO_HOME forbids accidental host configuration. Path
            # resolution requires no registry cache and does not download anything.
            environment["CARGO_HOME"] = str(private / "cargo-home")
            environment["CARGO_TARGET_DIR"] = str(private / "target")
            for ancestor in (private, *private.parents):
                if any((ancestor / ".cargo" / name).exists() for name in ("config", "config.toml")):
                    raise Ineligible("private metadata has ambient ancestor Cargo configuration")
            version = subprocess.run(
                [cargo, "-Vv"],
                cwd=private,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )
            if version.returncode or not version.stdout.startswith("cargo 1.92.0 "):
                raise Ineligible(
                    "resolution requires audited pinned Cargo 1.92.0: " + version.stdout.strip()
                )
            completed = subprocess.run(
                command, cwd=private, env=environment, capture_output=True, text=True, check=False
            )
            self.evidence = {
                "command": [arg.replace(str(private), "<private-resolution>") for arg in command],
                "exit_code": completed.returncode,
                "stderr": completed.stderr,
                "cargo_identity": version.stdout,
                "mirror_policy": (
                    "selected-member-only; normal path edges; stub sources; "
                    "no dev edges; Cargo locked/offline"
                ),
            }
            if completed.returncode:
                raise Ineligible(
                    "private locked Cargo resolution failed: " + completed.stderr.strip()
                )
            metadata = json.loads(completed.stdout)
            reverse = {
                str(private / value / "Cargo.toml"): key for key, value in identities.items()
            }
            package_keys = {
                package["id"]: reverse[package["manifest_path"]] for package in metadata["packages"]
            }
            nodes = {package_keys[node["id"]]: node for node in metadata["resolve"]["nodes"]}
            active = set()

            def visit(key: str) -> None:
                if key in active:
                    return
                active.add(key)
                for edge in nodes[key]["deps"]:
                    visit(package_keys[edge["pkg"]])

            visit(selected)
            for key in active:
                for feature in nodes[key]["features"]:
                    if any(
                        "?/" in token
                        for token in self.manifests[key].get("features", {}).get(feature, [])
                    ):
                        raise Ineligible(
                            f"{key}: active weak optional feature forwarding unsupported; "
                            "Cargo metadata over-activates it"
                        )
            return {
                key: {
                    "features": sorted(nodes[key]["features"]),
                    "edges": sorted(
                        [
                            {"name": edge["name"], "package": package_keys[edge["pkg"]]}
                            for edge in nodes[key]["deps"]
                        ],
                        key=lambda edge: edge["name"],
                    ),
                }
                for key in sorted(active)
            }

    def inventory(self, key: str) -> dict:
        root = self.roots[key]
        raw = self.manifests[key]
        library = raw.get("lib", {})
        if raw["package"].get("build") not in (None, False) or (root / "build.rs").exists():
            raise Ineligible(f"{key}: build script unsupported")
        if library.get("proc-macro"):
            raise Ineligible(f"{key}: proc macro unsupported")
        source = root / library.get("path", "src/lib.rs")
        if not _under(source.resolve(), root) or not source.is_file():
            raise Ineligible(f"{key}: library source missing or outside package")
        if source.relative_to(root).parts[0] in {"target", ".git"}:
            raise Ineligible(f"{key}: output/VCS directory used as source root")
        files = []
        directories = []
        for current, dirnames, filenames in os.walk(root, followlinks=False):
            current_path = Path(current)
            if current_path == root:
                dirnames[:] = [name for name in dirnames if name not in {"target", ".git"}]
            directories.append(current_path.relative_to(root).as_posix())
            for name in sorted(dirnames + filenames):
                path = current_path / name
                if path.is_symlink():
                    raise Ineligible(f"{key}: source symlink unsupported: {path.relative_to(root)}")
            dirnames.sort()
            for name in sorted(filenames):
                path = current_path / name
                if not path.is_file():
                    raise Ineligible(f"{key}: non-regular source input {name}")
                if path == root / "Cargo.toml":
                    continue  # reviewed semantic projection, not raw manifest bytes
                data = self.read(path)
                relative = path.relative_to(root).as_posix()
                if path.suffix == ".rs":
                    self.audit_source(key, path, data)
                files.append(
                    {
                        "path": relative,
                        "sha256": hashlib.sha256(data).hexdigest(),
                        "bytes": len(data),
                    }
                )
        return {
            "directories": sorted(directories),
            "files": files,
            "exclusions": ["top-level target/", "top-level .git/"],
        }

    def audit_source(self, key: str, path: Path, data: bytes) -> None:
        text = _without_comments(data.decode("utf-8"))
        root = self.roots[key]
        if re.search(r"#\s*\[\s*cfg_attr\b[^]]*\blink\s*\(", text):
            raise Ineligible(f"{key}: conditional native link attribute unsupported in {path.name}")
        for native in re.finditer(r"#\s*\[\s*link\s*\(([^]]*)\)\s*\]", text):
            attributes = {}
            for field in native.group(1).strip().rstrip(",").split(","):
                match = re.fullmatch(r'\s*([a-z_]+)\s*=\s*"([^"\\]*)"\s*', field)
                if not match or match[1] in attributes:
                    raise Ineligible(f"{key}: native library attribute unsupported in {path.name}")
                attributes[match[1]] = match[2]
            kind = attributes.get("kind", "dylib")
            if (
                not attributes.get("name")
                or set(attributes) - {"name", "kind", "modifiers"}
                or kind not in {"dylib", "raw-dylib"}
                or (
                    "modifiers" in attributes
                    and not (kind == "raw-dylib" and attributes["modifiers"] == "+verbatim")
                )
            ):
                raise Ineligible(f"{key}: native library input unsupported in {path.name}")
            # Dylib attributes defer resolution to the final consumer, unlike
            # bundled static libraries. The pinned-rustc negative-control test
            # proves a deliberately absent dylib does not become a body input.
        if re.search(r"\.(?:incbin|include)\s", text):
            raise Ineligible(f"{key}: assembly external input unsupported in {path.name}")
        # Deliberately conservative textual inspection also sees comments and
        # inactive cfg branches. Nonliteral includes are unsupported.
        for match in re.finditer(
            r"\b(include|include_str|include_bytes)\s*!\s*[({\[]([^)}\]]*)[)}\]]", text
        ):
            value = match.group(2).strip()
            if not re.fullmatch(r'"[^"\\]*"\s*,?', value):
                raise Ineligible(f"{key}: computed source include unsupported in {path.name}")
            target = (path.parent / value.rstrip(",").strip()[1:-1]).resolve()
            if (
                not _under(target, root)
                or target.name in {"Cargo.toml", "Cargo.lock"}
                or "target" in target.relative_to(root).parts
                or ".git" in target.relative_to(root).parts
            ):
                raise Ineligible(
                    f"{key}: external/manifest/output include unsupported in {path.name}"
                )
        for match in re.finditer(r'\bpath\s*=\s*"([^"\\]*)"', text):
            if not _under((path.parent / match.group(1)).resolve(), root):
                raise Ineligible(f"{key}: external module path unsupported in {path.name}")
        for match in re.finditer(r"\b(?:env|option_env)\s*!\s*[({\[]([^)}\]]*)[)}\]]", text):
            literal = re.match(r'\s*"([^"\\]*)"', match.group(1))
            if not literal:
                raise Ineligible(f"{key}: computed environment observation unsupported")
            name = literal.group(1)
            if name not in PACKAGE_ENV and name not in self.context.get("environment", {}):
                raise Ineligible(f"{key}: environment observation not declared: {name}")


def discover(
    repo_root: Path,
    body: str,
    context: dict | None = None,
    *,
    features: tuple[str, ...] = (),
    default_features: bool = True,
    profile: str = "release",
    force_staticlib: bool = False,
    cargo: str = "cargo",
    workspace: Path | None = None,
) -> dict:
    """Read and project one body. Ineligibility is returned, never silent fallback.

    context supplies actual target bytes, effective ordered flags/configuration,
    compiler+Cargo+sysroot identity, explicit environment (None != empty), and
    execution policy. Missing context permits a source projection, never action_key.
    This is discovery evidence only; snapshot/isolation/acceptance remains owed.
    """
    started = time.monotonic()
    repo = Path(repo_root).resolve()
    ws = Path(workspace).resolve() if workspace else repo / "sdk/lib/rust"
    context = context or {}
    state = Discovery(repo, ws, context)
    result: dict = {
        "schema": SCHEMA,
        "eligible": False,
        "reasons": [],
        "action_key": None,
        "source_projection_key": None,
        "whole_manifest_key": None,
        "descriptor": None,
    }
    try:
        workspace_data = state.read(ws / "Cargo.toml")
        root = tomllib.loads(workspace_data.decode())
        _check_keys(root, {"workspace", "profile"}, "workspace manifest")
        _check_keys(
            root.get("workspace", {}),
            {
                "members",
                "exclude",
                "default-members",
                "resolver",
                "package",
                "dependencies",
                "metadata",
            },
            "workspace",
        )
        resolver = root.get("workspace", {}).get("resolver", "1")
        if resolver != "2":
            raise Ineligible("only explicit Cargo resolver 2 is audited")
        selected = state.load(ws / body / "Cargo.toml")
        names = [raw["package"]["name"] for raw in state.manifests.values()]
        if len(set(names)) != len(names):
            raise Ineligible("duplicate package names/version disambiguation unsupported")
        lock_data = state.read(ws / "Cargo.lock")
        lock = tomllib.loads(lock_data.decode())
        if lock.get("version") not in {3, 4}:
            raise Ineligible("unsupported Cargo lock schema")
        resolved = state.metadata(
            selected, tuple(sorted(set(features))), default_features, cargo, resolver
        )
        lock_projection = []
        for key, node in resolved.items():
            package = state.manifests[key]["package"]
            candidates = [
                entry
                for entry in lock.get("package", [])
                if entry.get("name") == package["name"]
                and entry.get("version") == package["version"]
                and "source" not in entry
            ]
            if len(candidates) != 1:
                raise Ineligible(f"{key}: existing lock does not pin the path package uniquely")
            entry = candidates[0]
            for edge in node["edges"]:
                child = state.manifests[edge["package"]]["package"]
                if not any(
                    dependency == child["name"]
                    or dependency.startswith(child["name"] + " " + child["version"])
                    for dependency in entry.get("dependencies", [])
                ):
                    raise Ineligible(
                        f"{key}: existing lock is missing resolved edge to {child['name']}"
                    )
            lock_projection.append(
                {
                    "name": package["name"],
                    "version": package["version"],
                    "source": "path:" + key,
                    "edges": node["edges"],
                }
            )
        if profile not in {"release", "dev"}:
            raise Ineligible("only built-in release/dev profiles are audited")
        selected_profile = root.get("profile", {}).get(profile, {})
        _check_keys(
            selected_profile, PROFILE_KEYS | {"package", "build-override"}, f"profile.{profile}"
        )
        overrides = selected_profile.get("package", {})
        if "*" in overrides:
            raise Ineligible("wildcard profile override workspace membership semantics unsupported")
        projected_profile = {
            key: value for key, value in selected_profile.items() if key in PROFILE_KEYS
        }
        projected_profile["package"] = {}
        for key in resolved:
            name = state.manifests[key]["package"]["name"]
            if name in overrides:
                _check_keys(overrides[name], PROFILE_KEYS, f"profile.{profile}.package.{name}")
                projected_profile["package"][name] = overrides[name]
        # Version-qualified package override selectors may influence this graph.
        for selector in overrides:
            if selector not in names and any(
                selector.startswith(name + ":") or selector.startswith(name + "@") for name in names
            ):
                raise Ineligible("version-qualified profile package override unsupported")
        packages = {}
        for key, node in resolved.items():
            raw = state.manifests[key]
            packages[key] = {
                "package": raw["package"],
                "lib": raw.get("lib", {}),
                "features": node["features"],
                "edges": node["edges"],
                "inventory": state.inventory(key),
            }
        descriptor = {
            "schema": SCHEMA,
            "request": {
                "package": selected,
                "features": sorted(set(features)),
                "default_features": default_features,
                "profile": profile,
                "invocation": (
                    "cargo rustc --lib --crate-type staticlib"
                    if force_staticlib
                    else "cargo build --lib"
                ),
            },
            "resolver": resolver,
            "resolution_cargo": state.evidence["cargo_identity"],
            "profiles": projected_profile,
            "packages": packages,
            "lock": lock_projection,
        }
        result["source_projection_key"] = digest(descriptor)
        result["descriptor"] = {**descriptor, "context": context}
        result["whole_manifest_key"] = digest(
            {
                "source_projection": descriptor,
                "workspace_manifest": hashlib.sha256(workspace_data).hexdigest(),
                "lockfile": hashlib.sha256(lock_data).hexdigest(),
                "package_manifests": {
                    key: hashlib.sha256(value.encode()).hexdigest()
                    for key, value in sorted(state.raw_manifests.items())
                },
                "context": context,
            }
        )
        result["eligible"] = True
        missing = sorted(CONTEXT_KEYS - set(context))
        if not missing:
            result["action_key"] = digest(result["descriptor"])
        result["missing_context"] = missing
        result["files"] = sum(len(package["inventory"]["files"]) for package in packages.values())
        result["bytes"] = sum(
            file["bytes"] for package in packages.values() for file in package["inventory"]["files"]
        )
        result["package_count"] = len(packages)
    except (Ineligible, OSError, ValueError, KeyError, TypeError) as error:
        result["eligible"] = False
        result["action_key"] = None
        result["reasons"].append(str(error))
    result["provenance"] = {
        "reads": state.reads,
        "resolution": state.evidence,
        "elapsed_seconds": time.monotonic() - started,
        "identity_status": "projected live inputs; no immutable snapshot or executor authorization",
        "normalization": "repository-relative package roots; private resolution paths excluded",
        "limitations": [
            "normal path dependencies only; no target/build/registry/git edges",
            "build --lib only; no scripts/proc macros/lints/inherited workspace fields",
            "pinned Cargo 1.92.0; resolver 2; no active weak optional forwarding; "
            "release/dev; no wildcard/version-qualified profile overrides",
            "source text audit is conservative, not an execution read sandbox",
            "caller must supply complete actual execution context; "
            "discovery does not establish toolchain identity",
            "live discovery cannot establish snapshot coherence against concurrent writers",
        ],
    }
    return result
