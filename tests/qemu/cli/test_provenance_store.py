"""A body the artifact store served is judged by its store reference, not a stale copy."""

import json
import os
from pathlib import Path

from . import provenance


def _ref(store: Path, repo: Path, consumer: str, mtime: float) -> Path:
    refs = store / "v1/refs"
    refs.mkdir(parents=True, exist_ok=True)
    path = refs / f"{len(list(refs.iterdir()))}.json"
    path.write_text(json.dumps({"lane": str(repo.resolve()), "consumer": consumer}))
    os.utime(path, (mtime, mtime))
    return path


def test_the_lane_s_newest_ref_for_the_body_and_arch_is_cargo_s_output(tmp_path, monkeypatch):
    repo, store = tmp_path / "repo", tmp_path / "store"
    crate = repo / "sdk/lib/rust/ntdll-rust"
    (crate / "src").mkdir(parents=True)
    (crate / "src/lib.rs").write_text("")
    os.utime(crate / "src/lib.rs", (2000, 2000))
    old = repo / "sdk/lib/rust/target/body-storage-v1-arm64-x-0-rustlib/r/release/libntdll_rust.a"
    old.parent.mkdir(parents=True)
    old.write_bytes(b"!<arch>\n")
    os.utime(old, (1000, 1000))  # a private copy from before the store served it
    build = repo / "build-arm64"
    build.mkdir()
    monkeypatch.setenv("VIBEOS_ARTIFACT_STORE_ROOT", str(store))
    out = "/tmp/vibeos/sdk/lib/rust/target/cell-link-v1-arm64-3/x/release/build/n/out"
    ref = _ref(store, repo, f"cargo:ntdll-cargo|{out}|ntdll-rust|[]|true|false", 3000)
    _ref(
        store,
        repo,
        f"cargo:ntdll-cargo|{out.replace('arm64', 'amd64')}|ntdll-rust|[]|true|false",
        4000,
    )
    _ref(store, tmp_path / "other-lane", f"cargo:ntdll-cargo|{out}|ntdll-rust|[]|true|false", 5000)
    cargo_out, _copied = provenance._crate_staticlibs(repo, build, crate)
    assert cargo_out == ref
    # A source edit after the build's last request is stale, as with a private archive.
    assert cargo_out.stat().st_mtime > (crate / "src/lib.rs").stat().st_mtime
    os.utime(crate / "src/lib.rs", (3500, 3500))
    assert cargo_out.stat().st_mtime < (crate / "src/lib.rs").stat().st_mtime
