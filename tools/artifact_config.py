"""Fail-loud audit of the original nested Cargo's ambient configuration.

The isolated recipe deliberately supplies no Cargo configuration. Only proven
inert target settings may be omitted. A bounded supervisor scan checks missing
config membership before and after every outer build, even when Cargo is fresh;
never recursively watch '/' or a home directory through Cargo.
"""

from __future__ import annotations

import hashlib
import os
import re
import stat
from pathlib import Path

try:
    import tomllib  # type: ignore[reportMissingImports]  # Python 3.11+; fallback below.
except ImportError:
    import tomli as tomllib

TARGET = "x86_64-pc-windows-gnu"
CANONICAL_ROOT = Path("/tmp/vibeos")
IRRELEVANT_TARGETS = {"i686-pc-windows-gnu", "aarch64-pc-windows-gnullvm", "i686-kernel"}
SCRUBBED_BUILD_ENV = {"CARGO_BUILD_TARGET", "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_RUSTC"}
SCRUBBED_UNSTABLE_ENV = {"CARGO_UNSTABLE_BUILD_STD", "CARGO_UNSTABLE_BUILD_STD_FEATURES"}


class ConfigError(ValueError):
    """The original invocation has an ambient input outside the audited recipe."""


def cargo_scope(repo: Path) -> list[Path]:
    repo = repo.resolve()
    workspace = repo / "sdk/lib/rust"
    paths = {repo / "build-amd64", workspace}
    if workspace.is_dir():
        paths.update(
            path
            for path in workspace.iterdir()
            if path.name not in {"target", ".git", ".cargo"}
            and path.is_dir()
            and not path.is_symlink()
        )
    return sorted(paths)


def mapped_cwd(repo: Path, cargo_cwd: Path | None, *, kernel: bool) -> Path:
    repo = repo.resolve()
    cwd = (
        cargo_cwd if cargo_cwd is not None else repo / ("sdk/lib/rust" if kernel else "build-amd64")
    )
    cwd = Path(cwd)
    if not cwd.is_absolute():
        raise ConfigError("actual nested Cargo cwd must be absolute")
    cwd = cwd.resolve()
    if cwd == CANONICAL_ROOT or CANONICAL_ROOT in cwd.parents:
        try:
            aliases_repo = os.path.samefile(CANONICAL_ROOT, repo)
        except FileNotFoundError:
            aliases_repo = False
        if not aliases_repo:
            raise ConfigError("canonical Cargo cwd does not mount this source checkout")
        cwd = repo / cwd.relative_to(CANONICAL_ROOT)
    if cwd not in cargo_scope(repo):
        raise ConfigError(f"nested Cargo cwd is outside the supervisor's audited scope: {cwd}")
    return cwd


def _environment(environment: dict, broker: Path | None) -> None:
    for key, value in environment.items():
        if key.startswith("CARGO_PROFILE_"):
            raise ConfigError(f"unsupported original Cargo profile environment: {key}")
        if key.startswith("CARGO_TARGET_") and key.endswith("_RUSTFLAGS"):
            raise ConfigError(f"unsupported original Cargo target flags environment: {key}")
        if key == "RUSTC_BOOTSTRAP" and value:
            raise ConfigError("unsupported original compiler environment: RUSTC_BOOTSTRAP")
        if key.startswith("CARGO_UNSTABLE_") and key not in SCRUBBED_UNSTABLE_ENV:
            raise ConfigError(f"unsupported original Cargo unstable environment: {key}")
        if not key.startswith("CARGO_BUILD_") or key in SCRUBBED_BUILD_ENV:
            continue
        if key == "CARGO_BUILD_JOBS" and re.fullmatch(r"[1-9][0-9]*", value):
            continue
        if key == "CARGO_BUILD_RUSTC_WRAPPER":
            if not value:
                continue
            try:
                if broker is not None and os.path.samefile(value, broker):
                    continue
            except (FileNotFoundError, OSError):
                pass
        raise ConfigError(f"unsupported original Cargo build environment: {key}")


def _config_bytes(path: Path) -> bytes:
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    with os.fdopen(fd, "rb") as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode):
            raise ConfigError(f"Cargo configuration is not a regular file: {path}")
        data = stream.read()
        after = os.fstat(stream.fileno())

    def stamp(value):
        return (
            value.st_dev,
            value.st_ino,
            value.st_size,
            value.st_mtime_ns,
            value.st_ctime_ns,
        )

    if stamp(before) != stamp(after) or stamp(after) != stamp(path.lstat()):
        raise ConfigError(f"Cargo configuration changed during audit: {path}")
    return data


def _allowed(config: dict, path: Path) -> None:
    if set(config) - {"target"}:
        raise ConfigError(f"unsupported original Cargo configuration section: {path}")
    targets = config.get("target", {})
    if not isinstance(targets, dict):
        raise ConfigError(f"invalid Cargo target configuration: {path}")
    for target, settings in targets.items():
        if not isinstance(settings, dict):
            raise ConfigError(f"invalid Cargo target table: {path}: {target}")
        if target == TARGET:
            if set(settings) - {"linker"} or (
                "linker" in settings and not isinstance(settings["linker"], str)
            ):
                raise ConfigError(f"unsupported original amd64 Cargo target configuration: {path}")
        elif target in IRRELEVANT_TARGETS:
            if set(settings) - {"linker", "runner", "rustflags", "rustdocflags"}:
                raise ConfigError(f"unaudited Cargo target configuration: {path}: {target}")
            for key, value in settings.items():
                if not isinstance(value, str) and not (
                    key != "linker"
                    and isinstance(value, list)
                    and all(isinstance(item, str) for item in value)
                ):
                    raise ConfigError(f"invalid Cargo target setting: {path}: {target}.{key}")
        else:
            raise ConfigError(f"unaudited Cargo target selector: {path}: {target}")


def _audit(repo: Path, cwds: list[Path], *, environment: dict | None, broker: Path | None) -> dict:
    env = dict(os.environ if environment is None else environment)
    _environment(env, broker)
    directories = set()
    for cwd in cwds:
        directories.update(path / ".cargo" for path in (cwd, *cwd.parents))
    # The canonical mount may not exist in the supervisor namespace yet. Its
    # outside ancestors can still affect every future canonical child Cargo.
    directories.update(path / ".cargo" for path in CANONICAL_ROOT.parents)
    home = Path(env.get("CARGO_HOME", str(Path(env.get("HOME", str(Path.home()))) / ".cargo")))
    if not home.is_absolute():
        raise ConfigError("effective CARGO_HOME must be absolute for the pilot")
    directories.add(home)
    files = {}
    checked = []
    for directory in sorted(directories):
        if directory.is_symlink():
            raise ConfigError(f"symlinked Cargo configuration directory is unaudited: {directory}")
        for name in ("config", "config.toml"):
            path = directory / name
            checked.append(str(path))
            if not path.exists() and not path.is_symlink():
                continue
            if path.is_symlink():
                raise ConfigError(f"symlinked Cargo configuration is unaudited: {path}")
            payload = _config_bytes(path)
            try:
                _allowed(tomllib.loads(payload.decode("utf-8")), path)
            except (UnicodeError, tomllib.TOMLDecodeError) as error:
                raise ConfigError(
                    f"invalid original Cargo configuration: {path}: {error}"
                ) from error
            files[str(path)] = {
                "sha256": hashlib.sha256(payload).hexdigest(),
                "bytes": len(payload),
            }
    return {
        "scope_directories": len(cwds),
        "checked_paths": checked,
        "files": files,
        "watches": sorted(files),
    }


def audit_scope(repo: Path, *, environment: dict | None = None, broker: Path | None = None) -> dict:
    """Bounded preflight/postcheck, including additions when no request executes."""
    return _audit(repo, cargo_scope(repo), environment=environment, broker=broker)


def audit_request(
    repo: Path,
    *,
    cargo_cwd: Path | None = None,
    kernel: bool = False,
    environment: dict | None = None,
    broker: Path | None = None,
) -> dict:
    cwd = mapped_cwd(repo, cargo_cwd, kernel=kernel)
    return {"mapped_cwd": str(cwd), **_audit(repo, [cwd], environment=environment, broker=broker)}
