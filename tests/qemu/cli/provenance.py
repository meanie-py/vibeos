"""Compatibility DTOs and routing for native build/media provenance."""

from __future__ import annotations

import functools
import os
from dataclasses import asdict, dataclass, field
from pathlib import Path

from tools import build_infra_compat as native

from . import archcfg

STAMP_NAME = ".vibeos-build-stamp.json"
_RGLOB_RETRIES = 5


class ProvenanceError(RuntimeError):
    """The freshness oracle could not verify its required evidence."""


def _cfg(arch: str) -> dict:
    return asdict(archcfg.arch_build(arch))


def _call(op: str, *, error_type: type[Exception] = ProvenanceError, **args):
    args.setdefault("env", dict(os.environ))
    args.setdefault(
        "store_root",
        os.environ.get("VIBEOS_ARTIFACT_STORE_ROOT") or str(Path.home() / "vibeos-artifact-store"),
    )
    return native.invoke("provenance_" + op, args, root=Path.cwd(), error_type=error_type)


def __getattr__(name):
    if name == "NON_BUILD_PATHSPEC":
        return tuple(_call("non_build_pathspec"))
    raise AttributeError(name)


def _path(value):
    return Path(value) if value is not None else None


def _paths(values):
    return [Path(value) for value in values]


def _run_bytes(cmd, cwd, *, ok_codes=(0,), env=None):
    return bytes(
        _call(
            "run_bytes",
            cmd=cmd,
            cwd=cwd,
            ok_codes=ok_codes,
            env=dict(os.environ) if env is None else env,
        )
    )


def _run(cmd, cwd, *, ok_codes=(0,)):
    return _run_bytes(cmd, cwd, ok_codes=ok_codes).decode("utf-8", "replace")


def git_head(repo):
    return _call("git_head", repo=repo)


def git_fingerprint(repo, paths=None):
    return _call("git_fingerprint", repo=repo, paths=paths)


def tree_content_hash(repo, paths):
    return _call("tree_content_hash", repo=repo, paths=paths)


def _sha256_file(p):
    return _call("sha256_file", path=p)


def rglob_stable(root, pattern):
    return _paths(_call("rglob_stable", path=root, pattern=pattern, error_type=OSError))


def discover_rust_targets(build_dir):
    return _paths(_call("discover_rust_targets", build_dir=build_dir, error_type=OSError))


def discover_rust_archives(build_dir):
    return _paths(_call("discover_rust_archives", build_dir=build_dir, error_type=OSError))


def workspace_cargo_target(repo, arch):
    return _path(_call("workspace_cargo_target", repo=repo, arch=arch))


def workspace_cargo_targets(repo, arch):
    return _paths(_call("workspace_cargo_targets", repo=repo, arch=arch, error_type=ValueError))


def discover_rust_module_outputs(repo, arch):
    return _paths(_call("discover_rust_module_outputs", repo=repo, arch=arch, cfg=_cfg(arch)))


@functools.cache
def _workspace_members(repo_str):
    value = _call("workspace_members", repo=repo_str)
    return frozenset(value) if value is not None else None


@functools.lru_cache(maxsize=4)
def _cmake_named_crates(repo):
    value = _call("cmake_named_crates", repo=repo)
    return frozenset(value) if value is not None else None


def _crate_for_stem(repo, stem):
    return _path(_call("crate_for_stem", repo=repo, stem=stem))


def _crate_for_rel(repo, rel):
    return _path(_call("crate_for_rel", repo=repo, rel=rel))


def _newest_rs_mtime(crate):
    return _call("newest_rs_mtime", crate=crate)


def _depinfo_rs_mtime(repo, cargo_out, crate):
    return _call("depinfo_rs_mtime", repo=repo, cargo_out=cargo_out, crate=crate)


def mkramdisk_required_rels(repo):
    return _call("mkramdisk_required_rels", repo=repo)


def boot_set_cohab_rels(repo, arch):
    return _call("boot_set_cohab_rels", repo=repo, arch=arch, cfg=_cfg(arch))


def cohab_staleness_reasons(repo, arch):
    return _call("cohab_staleness_reasons", repo=repo, arch=arch, cfg=_cfg(arch))


def _belongs_to_sibling(copied, artifact):
    return _call("belongs_to_sibling", copied=copied, artifact=artifact)


def _lag(older, newer):
    return _call("lag", older=older, newer=newer)


class _StaticlibIndex:
    """Compatibility snapshot of the native per-check archive/reference index."""

    def __init__(self, repo, build_dir):
        value = _call("staticlib_index", repo=repo, build_dir=build_dir)
        self.libs = {name: _paths(paths) for name, paths in value["libs"].items()}
        self.refs = {body: Path(path) for body, path in value["refs"].items()}


def _crate_staticlibs(repo, build_dir, crate, index=None):
    raw_index = None if index is None else {"libs": index.libs, "refs": index.refs}
    value = _call("crate_staticlibs", repo=repo, build_dir=build_dir, crate=crate, index=raw_index)
    return tuple(_path(p) for p in value)


def _store_ref(repo, arch, body):
    return _path(_call("store_ref", repo=repo, arch=arch, body=body))


def _store_refs(repo, arch):
    return {body: Path(path) for body, path in _call("store_refs", repo=repo, arch=arch).items()}


def cargo_module_rels(repo):
    return _call("cargo_module_rels", repo=repo)


def _cargo_link_inputs(repo, build_dir, artifact):
    return _paths(_call("cargo_link_inputs", repo=repo, build_dir=build_dir, artifact=artifact))


def _media_manifest_rels(repo, arch):
    value = _call("media_manifest_rels", repo=repo, arch=arch, cfg=_cfg(arch))
    return set(value) if value is not None else None


def cargo_link_staleness_reasons(repo, arch):
    return _call("cargo_link_staleness_reasons", repo=repo, arch=arch, cfg=_cfg(arch))


@dataclass
class Stamp:
    arch: str
    ts: int
    git_fingerprint: str
    rust_src_hash: str
    boot_media: str
    boot_media_sha: str
    cold: bool
    contributing: dict = field(default_factory=dict)
    rust_targets: list = field(default_factory=list)
    build_src_hash: str = ""
    head_sha: str = ""
    rust_tree_hash: str = ""
    build_tree_hash: str = ""

    def write(self, build_dir):
        return Path(_call("stamp_write", build_dir=build_dir, stamp=asdict(self)))

    @staticmethod
    def read(build_dir):
        value = _call("stamp_read", build_dir=build_dir)
        return Stamp(**value) if value is not None else None


def rust_src_hash(repo):
    return _call("rust_src_hash", repo=repo)


def build_src_hash(repo):
    return _call("build_src_hash", repo=repo)


def contributing_hashes(repo, arch):
    return _call("contributing_hashes", repo=repo, arch=arch, cfg=_cfg(arch))


def make_stamp(repo, arch, cold, ts):
    return Stamp(**_call("make_stamp", repo=repo, arch=arch, cold=cold, ts=ts, cfg=_cfg(arch)))


def _src_delta(repo, stamp):
    return _call("src_delta", repo=repo, stamp=asdict(stamp))


@dataclass
class Staleness:
    fresh: bool
    reasons: list
    warm: bool
    media_missing: bool

    @property
    def stale(self):
        return not self.fresh or self.media_missing


def external_media_problems(repo, arch, image):
    return _call("external_media_problems", repo=repo, arch=arch, image=image)


def check_staleness(repo, arch):
    return Staleness(**_call("check_staleness", repo=repo, arch=arch, cfg=_cfg(arch)))


def _check_staleness(repo, arch):
    return Staleness(**_call("check_staleness_inner", repo=repo, arch=arch, cfg=_cfg(arch)))


def media_paths(repo, arch):
    return _paths(_call("media_paths", repo=repo, arch=arch, cfg=_cfg(arch)))


def _count_in_file(path, needles):
    counts = _call("count_in_file", path=path, needles=needles)
    return dict(zip(needles, counts, strict=True))


def probes_missing_from_media(repo, arch, literals):
    return _call(
        "probes_missing_from_media", repo=repo, arch=arch, literals=literals, cfg=_cfg(arch)
    )


def _mtime_str(p):
    return _call("mtime_str", path=p)
