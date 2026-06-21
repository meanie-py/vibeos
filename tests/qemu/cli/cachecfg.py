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

First-landing scope (opt-in via `--cache` / `VIBEOS_BUILD_CACHE=1`):
  * **ccache for C/C++** — cross-worktree hits via `CCACHE_BASEDIR` + `nohashdir`
    (automatic path rewriting; the ReactOS tree already wires `RULE_LAUNCH_COMPILE
    ccache` behind `-DENABLE_CCACHE`, which `build.py` toggles on).
  * **sccache for Rust** — caches rustc across the smoke harness's target-dir
    nuke (`RUSTC_WRAPPER` + `CARGO_INCREMENTAL=0`), **same-worktree**.

Deliberately NOT done yet (see plan §"path normalization"): the
`--remap-path-prefix` that would give cross-*worktree* Rust hits. This project
sets load-bearing per-target `rustflags` in `sdk/lib/rust/.cargo/config.toml`
(the i386 `-sse,-sse2` kernel guard); the `RUSTFLAGS` env var *replaces* those
rather than merging (cargo uses env-flags XOR config-flags), so injecting a remap
that way would silently put SSE back in the i386 kernel. Cross-worktree Rust is a
follow-up that must thread the remap through config.toml, not the env.
"""

from __future__ import annotations

import json
import os
import shutil
from dataclasses import dataclass
from pathlib import Path

DEFAULT_MAXSIZE = "50G"
_TRUEY = ("1", "true", "yes", "on")


def vibeos_root(repo: Path) -> Path:
    """The worktree-parent dir that owns caches. `VIBEOS_ROOT` env wins; else the
    repo's parent (siblings = the other worktrees), which gives every worktree the
    SAME basedir → cross-worktree ccache hits."""
    env = os.environ.get("VIBEOS_ROOT")
    if env:
        return Path(env).expanduser().resolve()
    return repo.resolve().parent


def cache_enabled(args=None) -> bool:
    """Opt-in for the first landing: `--cache` flag (tri-state) or
    `VIBEOS_BUILD_CACHE=1`. Default off so a cache bug can't wedge every build."""
    flag = getattr(args, "cache", None) if args is not None else None
    if flag is not None:
        return bool(flag)
    return os.environ.get("VIBEOS_BUILD_CACHE", "").lower() in _TRUEY


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
        """The env overlay to merge onto os.environ for the ninja subprocess."""
        e: dict[str, str] = {}
        if self.use_ccache:
            e["CCACHE_DIR"] = str(self.ccache_dir)
            e["CCACHE_BASEDIR"] = str(self.root)  # abs paths under here hashed relative
            e["CCACHE_NOHASHDIR"] = "true"  # cwd not in hash → worktree-independent
            e["CCACHE_SLOPPINESS"] = "time_macros"  # tolerate __DATE__/__TIME__ for hits
            e["CCACHE_MAXSIZE"] = DEFAULT_MAXSIZE
        if self.use_sccache:
            e["RUSTC_WRAPPER"] = self.sccache_bin or "sccache"
            e["SCCACHE_DIR"] = str(self.sccache_dir)
            e["SCCACHE_CACHE_SIZE"] = DEFAULT_MAXSIZE
            e["CARGO_INCREMENTAL"] = "0"  # incremental + sccache don't mix
        return e

    def summary(self) -> str:
        bits = []
        bits.append(f"ccache→{self.ccache_dir}" if self.use_ccache else "ccache(absent)")
        bits.append(f"sccache→{self.sccache_dir}" if self.use_sccache else "sccache(absent)")
        return f"root={self.root}  " + "  ".join(bits)


# ── stats parsing (pure — the build-twice hit-rate is also a determinism gauge) ──


def parse_sccache_json(text: str) -> dict:
    """Reduce `sccache --show-stats --stats-format=json` to {requests,hits,misses,hit_rate}.

    sccache buckets hits/misses by language (a dict) and also exposes scalar
    `cache_hits`/`cache_misses` historically; tolerate both shapes."""
    try:
        d = json.loads(text)
    except (ValueError, TypeError):
        return {}
    s = d.get("stats", d)

    def _total(v) -> int:
        if isinstance(v, dict):
            inner = v.get("counts", v)  # {"counts": {"Rust": 12}} or {"Rust": 12}
            if not isinstance(inner, dict):
                return 0
            return sum(n for n in inner.values() if isinstance(n, int))
        return v if isinstance(v, int) else 0

    requests = _total(s.get("compile_requests", 0))
    hits = _total(s.get("cache_hits", 0))
    misses = _total(s.get("cache_misses", 0))
    denom = hits + misses
    return {
        "requests": requests,
        "hits": hits,
        "misses": misses,
        "hit_rate": (hits / denom) if denom else 0.0,
    }


def parse_ccache_stats(text: str) -> dict:
    """Reduce `ccache --print-stats` (machine-readable `key\\tvalue`) to
    {hits,misses,hit_rate}. hits = direct + preprocessed; misses = cache_miss.
    (Far more robust than scraping `-s`, whose layout shifts across 3.x/4.x and
    double-counts a `Primary storage` section.)"""
    vals: dict[str, int] = {}
    for line in text.splitlines():
        parts = line.split("\t")
        if len(parts) == 2 and parts[1].strip().isdigit():
            vals[parts[0].strip()] = int(parts[1].strip())
    hits = vals.get("direct_cache_hit", 0) + vals.get("preprocessed_cache_hit", 0)
    misses = vals.get("cache_miss", 0)
    denom = hits + misses
    return {"hits": hits, "misses": misses, "hit_rate": (hits / denom) if denom else 0.0}


def make(repo: Path) -> CacheEnv:
    root = vibeos_root(repo)
    return CacheEnv(
        root=root,
        ccache_dir=root / "ccache",
        sccache_dir=root / "sccache",
        ccache_bin=shutil.which("ccache"),
        sccache_bin=shutil.which("sccache"),
    )
