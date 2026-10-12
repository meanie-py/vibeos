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
