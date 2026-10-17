import json
import os
from pathlib import Path
from types import SimpleNamespace

from . import prune, trim

DAY = 86400.0
NOW = 1_000_000_000.0


def unit(
    prof: Path, pkg: str, h: str, *, age_days: float, features=("default",), rustc=1, kind="lib"
):
    """One cargo unit: its fingerprint dir plus the deps/ and build/ outputs it names."""
    crate = pkg.replace("-", "_")
    fp = prof / ".fingerprint" / f"{pkg}-{h}"
    fp.mkdir(parents=True)
    (fp / f"{kind}-{crate}.json").write_text(
        json.dumps(
            {
                "rustc": rustc,
                "features": json.dumps(list(features)),
                "target": 7,
                "profile": 3,
                "path": hash(pkg) & 0xFFFF,
                "deps": [],
                "compile_kind": 0,
            }
        )
    )
    deps = prof / "deps"
    deps.mkdir(exist_ok=True)
    outs = [fp, fp / f"{kind}-{crate}.json", deps / f"lib{crate}-{h}.rlib", deps / f"{crate}-{h}.d"]
    for p in outs[2:]:
        p.write_bytes(b"x" * 4096)
    t = NOW - age_days * DAY
    for p in outs:
        os.utime(p, (t, t))
    return outs


def age_tree(root: Path, days: float):
    t = NOW - days * DAY
    for parent, dirs, files in os.walk(root):
        for name in dirs + files:
            os.utime(os.path.join(parent, name), (t, t))
    os.utime(root, (t, t))


def paths(plan):
    return {p.name for p, _ in plan.paths}


def test_older_unit_with_same_identity_is_superseded(tmp_path):
    prof = tmp_path / "target/kernel-cargo-x-rustlib/x86_64-pc-windows-gnu/release"
    unit(prof, "ntkrnl-sys", "a" * 16, age_days=5, rustc=1)
    unit(prof, "ntkrnl-sys", "b" * 16, age_days=1, rustc=2)
    plan = trim.plan_target(tmp_path / "target", now=NOW, idle_hours=24)
    assert plan.superseded_units == 1
    assert paths(plan) == {
        f"ntkrnl-sys-{'a' * 16}",
        f"libntkrnl_sys-{'a' * 16}.rlib",
        f"ntkrnl_sys-{'a' * 16}.d",
    }


def test_feature_variants_are_both_live(tmp_path):
    # Host release/ legitimately holds crt-shared-rust twice, with different features.
    prof = tmp_path / "target/release"
    unit(prof, "crt-shared-rust", "a" * 16, age_days=9, features=("default",))
    unit(prof, "crt-shared-rust", "b" * 16, age_days=1, features=("cpp", "default"))
    assert trim.plan_target(tmp_path / "target", now=NOW, idle_hours=24).paths == []


def test_same_named_targets_of_different_packages_are_both_live(tmp_path):
    # Five difftest packages each own a test target called `diff`.
    prof = tmp_path / "target/release"
    unit(prof, "rtl-difftest", "a" * 16, age_days=9, kind="test-integration-test")
    unit(prof, "cnt-difftest", "b" * 16, age_days=1, kind="test-integration-test")
    assert trim.plan_target(tmp_path / "target", now=NOW, idle_hours=24).paths == []


def test_recent_superseded_unit_is_kept(tmp_path):
    # A lane flipping between two configurations within the idle window keeps both.
    prof = tmp_path / "target/release"
    unit(prof, "cc", "a" * 16, age_days=0.5, rustc=1)
    unit(prof, "cc", "b" * 16, age_days=0.1, rustc=2)
    assert trim.plan_target(tmp_path / "target", now=NOW, idle_hours=24).paths == []


def test_unreadable_fingerprint_is_never_judged(tmp_path):
    prof = tmp_path / "target/release"
    unit(prof, "cc", "a" * 16, age_days=5, rustc=1)
    unit(prof, "cc", "b" * 16, age_days=1, rustc=2)
    (prof / ".fingerprint" / f"cc-{'a' * 16}" / "lib-cc.json").write_text("{half-written")
    assert trim.plan_target(tmp_path / "target", now=NOW, idle_hours=24).paths == []


def test_idle_adhoc_triple_and_debug_trees_go_whole(tmp_path):
    root = tmp_path / "target"
    for name in ("x86_64-pc-windows-gnu", "debug", "i686-unknown-linux-gnu"):
        unit(root / name / "release", "foo", "a" * 16, age_days=3)
        age_tree(root / name, 3)
    unit(root / "aarch64-pc-windows-gnullvm" / "release", "foo", "a" * 16, age_days=0.2)
    age_tree(root / "aarch64-pc-windows-gnullvm", 0.2)
    # CMake-owned dirs are never ad-hoc, however old.
    for name in ("win32k-cargo-win32k_kernel_rust-rustlib", "release"):
        unit(root / name / "release", "foo", "a" * 16, age_days=30)
        age_tree(root / name, 30)
    plan = trim.plan_target(root, now=NOW, idle_hours=24)
    assert plan.adhoc_dirs == 3
    assert paths(plan) == {"x86_64-pc-windows-gnu", "debug", "i686-unknown-linux-gnu"}


def test_apply_removes_plan_and_spares_the_live_unit(tmp_path):
    wt = tmp_path / "wt"
    prof = wt / "sdk/lib/rust/target/release"
    old = unit(prof, "syn", "a" * 16, age_days=5, rustc=1)
    new = unit(prof, "syn", "b" * 16, age_days=2, rustc=2)
    plan = trim.plan_worktree(wt, now=NOW, idle_hours=24)
    freed, failures = trim.apply(wt, plan)
    assert failures == 0 and freed > 0
    assert not any(p.exists() for p in old)
    assert all(p.exists() for p in new)


def test_prune_trim_dry_run_deletes_nothing(tmp_path, monkeypatch, capsys):
    wt = tmp_path / "wt"
    prof = wt / "sdk/lib/rust/target/release"
    old = unit(prof, "syn", "a" * 16, age_days=5, rustc=1)
    unit(prof, "syn", "b" * 16, age_days=2, rustc=2)
    monkeypatch.setattr(prune, "git_worktrees", lambda _: [wt])
    monkeypatch.setattr(trim, "PORT_STATE_ROOT", tmp_path / "port")
    monkeypatch.setattr(trim.time, "time", lambda: NOW)
    rc = prune.prune(SimpleNamespace(trim=True, apply=False, idle_hours=0), wt)
    assert rc == 0
    assert "1 superseded units" in capsys.readouterr().out
    assert all(p.exists() for p in old)


def test_port_staging_trims_idle_targets_and_never_state(tmp_path):
    root = tmp_path / "port"
    for stem, days in (("idle", 3), ("busy", 0.2)):
        d = root / stem
        (d / "hand").mkdir(parents=True)
        (d / "state.json").write_text("{}")
        age_tree(d, 30)
        unit(d / "target-amd64" / "release", "foo", "a" * 16, age_days=days)
        age_tree(d / "target-amd64", days)
    plan = trim.plan_port_staging(root, now=NOW, idle_hours=24)
    assert [p for p, _ in plan.paths] == [root / "idle" / "target-amd64"]
    freed, failures = trim.apply(root, plan)
    assert failures == 0 and freed > 0
    assert not (root / "idle" / "target-amd64").exists()
    assert (root / "idle" / "state.json").exists() and (root / "idle" / "hand").is_dir()
    assert (root / "busy" / "target-amd64").exists()


def test_a_custom_target_whose_json_is_gone_goes_whole(tmp_path):
    rust = tmp_path / "sdk/lib/rust"
    target = rust / "target"
    for cell, has_json in (("old-cargo", False), ("kept-cargo", True)):
        (rust / cell).mkdir(parents=True)
        (rust / cell / "Cargo.toml").write_text("")
        if has_json:
            (rust / cell / "i686-kernel.json").write_text("{}")
        for d in (f"{cell}-i386-cargo-link", f"{cell}-body_rust-rustlib"):
            for sub in ("i686-kernel", "i686-pc-windows-gnu", "release"):
                (target / d / sub / "release").mkdir(parents=True)
    # A per-module dir no cell owns is not judged.
    (target / "gone-cargo-i386-cargo-link/i686-kernel").mkdir(parents=True)
    old = os.stat(target).st_mtime + 2 * 3600
    plan = trim.plan_worktree(tmp_path, now=old, idle_hours=24)
    dead = sorted(str(p.relative_to(target)) for p, _ in plan.paths)
    assert dead == [
        "old-cargo-body_rust-rustlib/i686-kernel",
        "old-cargo-i386-cargo-link/i686-kernel",
    ]
    assert plan.dead_targets == 2
    # Within the hour a build that began before the json went may still be writing it.
    assert trim.plan_worktree(tmp_path, now=os.stat(target).st_mtime + 60).dead_targets == 0


def test_retired_layouts_go_only_when_the_lane_s_own_config_retired_them(tmp_path):
    wt = tmp_path
    build = wt / "build-amd64"
    for d in ("base/live/rust-live_rust-target/x", "base/old/rust-old_rust-target/x"):
        (build / d).mkdir(parents=True)
    (build / "build.ninja").write_text(
        "build x: CUSTOM CARGO_TARGET_DIR=/tmp/vibeos/build-amd64/base/live/rust-live_rust-target\n"
    )
    target = wt / "sdk/lib/rust/target"
    (target / "beep-cargo-beep_rust-rustlib/x86_64-pc-windows-gnu").mkdir(parents=True)
    (target / "body-storage-v1-amd64-00-0-rustlib/x").mkdir(parents=True)
    later = os.stat(target).st_mtime + 2 * 3600

    def names(plan):
        return sorted(str(p.relative_to(wt)) for p, _ in plan.paths)

    # Body sharding not configured: the per-cell dir may still be the live layout.
    (build / "CMakeCache.txt").write_text("VIBEOS_RUST_BODY_SHARDS:STRING=0\n")
    assert names(trim.plan_worktree(wt, now=later, idle_hours=1)) == [
        "build-amd64/base/old/rust-old_rust-target"
    ]
    (build / "CMakeCache.txt").write_text("VIBEOS_RUST_BODY_SHARDS:STRING=4\n")
    plan = trim.plan_worktree(wt, now=later, idle_hours=1)
    assert names(plan) == [
        "build-amd64/base/old/rust-old_rust-target",
        "sdk/lib/rust/target/beep-cargo-beep_rust-rustlib",
    ]
    assert plan.retired_dirs == 2
    # Nothing recent goes.
    assert trim.plan_worktree(wt, now=os.stat(target).st_mtime + 60, idle_hours=1).paths == []


def test_per_cell_link_dirs_retire_once_their_arch_shares_link_dirs(tmp_path):
    wt = tmp_path
    build = wt / "build-amd64"
    build.mkdir()
    (build / "build.ninja").write_text("")
    target = wt / "sdk/lib/rust/target"
    for d in ("beep-cargo-amd64-cargo-link/release", "beep-cargo-i386-cargo-link/release"):
        (target / d).mkdir(parents=True)
    later = os.stat(target).st_mtime + 2 * 3600
    (build / "CMakeCache.txt").write_text("VIBEOS_CARGO_LINK_SHARDS:STRING=0\n")
    assert trim.plan_worktree(wt, now=later, idle_hours=1).paths == []
    (build / "CMakeCache.txt").write_text("VIBEOS_CARGO_LINK_SHARDS:STRING=24\n")
    plan = trim.plan_worktree(wt, now=later, idle_hours=1)
    # Only amd64's: i386 has no configured build saying it moved.
    assert [p.name for p, _ in plan.paths] == ["beep-cargo-amd64-cargo-link"]


def test_body_shards_retire_when_no_build_has_acquired_them_for_days(tmp_path):
    """Once the artifact store builds every eligible body, a shard only the moved bodies
    used is read by nothing. Its lock file's mtime (stamped on every acquire) is the
    last use; file mtimes inside cannot say, since cargo rewrites nothing fresh."""
    import fcntl

    target = tmp_path / "sdk/lib/rust/target"
    used = target / "body-storage-v1-amd64-ff00ca30fac39de6-2-rustlib"
    idle = target / "body-storage-v1-i386-17bf15d2b52d02b0-0-rustlib"
    held = target / "body-storage-v1-arm64-1e1531d0eeccac08-1-rustlib"
    for shard in (used, idle, held):
        (shard / "release/deps").mkdir(parents=True)
        (shard / ".body-storage.lock").write_text("")
    now = 2_000_000_000.0
    old = now - trim.BODY_SHARD_IDLE_SECONDS - 60
    for shard in (idle, held):
        os.utime(shard / ".body-storage.lock", (old, old))
    os.utime(used / ".body-storage.lock", (now - 3600, now - 3600))
    with open(held / ".body-storage.lock", "rb") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        plan = trim.plan_worktree(tmp_path, now=now)
    assert [p.name for p, _ in plan.paths] == [idle.name]


def test_idle_orchestrator_target_dirs_are_ad_hoc(tmp_path):
    target = tmp_path / "sdk/lib/rust/target"
    (target / "orchestrate-i386-276820ffca02/release").mkdir(parents=True)
    later = os.stat(target).st_mtime + 2 * 3600
    plan = trim.plan_worktree(tmp_path, now=later, idle_hours=1)
    assert [p.name for p, _ in plan.paths] == ["orchestrate-i386-276820ffca02"]
