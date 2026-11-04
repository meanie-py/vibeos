"""Build-cache env — the single owner of ccache/sccache configuration.

`vibeos-cli build` computes the cache env here from `VIBEOS_ROOT` and injects it
into the ninja subprocess, so no shell/agent has to configure caching itself
(plan: `docs/engineering/build-cache-and-determinism-plan.md`). One owner →
consistent across worktrees + agents, and the single place to flip
local↔distributed (S3) later.

`VIBEOS_ROOT` is the directory that **contains all worktrees** — that is exactly
what `CCACHE_BASEDIR` must be for a build in `vibeos-wt-X` to reuse objects
compiled in `vibeos`. Default: the repo's parent dir (which is that directory on
this machine, `~/Projects/os-playground`). Override with the `VIBEOS_ROOT` env
var; a machine that keeps everything under `~/.vibeos` points it there.

Scope (MANDATORY since 2026-07-03 — on unless `--no-cache` / `VIBEOS_BUILD_CACHE=0`):
  * **ccache for C/C++** — cross-worktree hits via `CCACHE_BASEDIR` + `nohashdir`
    (automatic path rewriting; the ReactOS tree already wires `RULE_LAUNCH_COMPILE
    ccache` behind `-DENABLE_CCACHE`, which `build.py` toggles on).
  * **sccache for Rust** — caches rustc across the smoke harness's target-dir
    nuke (`RUSTC_WRAPPER` + `CARGO_INCREMENTAL=0`), across canonical worktrees.

Cross-worktree path-independence (both languages) comes from the CANONICAL BUILD
PATH: `vibeos-cli build` runs cmake/ninja/cargo with the worktree bind-mounted at
/tmp/vibeos (see canon.py / tools/canon-run.sh), so compiler argv and embedded
paths are identical from every worktree — that's what lets ccache AND sccache
hit across worktrees with no remap flags. (The old det-remap
`--remap-path-prefix` approach was removed 2026-07-03: sccache marks such
invocations non-cacheable, which had silently disabled all Rust caching.)
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path

from tools import build_infra_compat as native

_constants = native.call("build_policy_constants", {})
CCACHE_MAXSIZE = _constants["ccache_maxsize"]
SCCACHE_MAXSIZE = _constants["sccache_maxsize"]


def _call(op, **arguments):
    return native.call("build_policy_" + op, arguments)


def vibeos_root(repo: Path) -> Path:
    return Path(_call("cache_root", repo=repo, environment=dict(os.environ)))


def cache_enabled(args=None) -> bool:
    flag = getattr(args, "cache", None) if args is not None else None
    return _call(
        "cache_enabled", flag=bool(flag) if flag is not None else None, environment=dict(os.environ)
    )


@dataclass(frozen=True)
class CacheEnv:
    root: Path
    ccache_dir: Path
    sccache_dir: Path
    ccache_bin: str | None
    sccache_bin: str | None

    @property
    def use_ccache(self) -> bool:
        return self.ccache_bin is not None

    @property
    def use_sccache(self) -> bool:
        return self.sccache_bin is not None

    def env(self) -> dict[str, str]:
        return _call("cache_env", cache=self)

    def summary(self) -> str:
        bits = []
        bits.append(f"ccache→{self.ccache_dir}" if self.use_ccache else "ccache(absent)")
        bits.append(f"sccache→{self.sccache_dir}" if self.use_sccache else "sccache(absent)")
        return f"root={self.root}  " + "  ".join(bits)


def parse_sccache_json(text: str) -> dict:
    return _call("sccache_stats", text=text)


def parse_ccache_stats(text: str) -> dict:
    return _call("ccache_stats", text=text)


def make(repo: Path) -> CacheEnv:
    result = _call("cache_make", repo=repo, environment=dict(os.environ))
    return CacheEnv(
        **{**result, **{k: Path(result[k]) for k in ("root", "ccache_dir", "sccache_dir")}}
    )
