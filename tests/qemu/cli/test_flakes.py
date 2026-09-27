"""flake signatures + flake-report grading. Every case here is a failure that a
pass/fail tally or a naive grader got wrong on the real fleet history."""

import json
import subprocess

from . import artifacts, flakes
from .artifacts import Run, Stage


def boot(cls, salient, **kw):
    return {"ok": False, "classification": cls, "salient": salient, **kw}


# --------------------------------------------------------------------- signatures


def test_bugcheck_signature_keeps_code_marker_and_function_not_offset():
    b = boot(
        "BUGCHECK",
        [
            "BugCheck 0x0000001E (after boot marker)",
            "hang: CPU#0 0xfffff80000462cfe -> 00462cfe\tKeRaiseIrql+14\tshim_exports.rs:956",
        ],
    )
    assert flakes.boot_signature(b) == "BUGCHECK 0x1E post-marker @ KeRaiseIrql"


def test_a_wrong_image_symbol_is_not_a_cluster_key():
    """`XHF_NOEXECUTE+314197` is a constant hundreds of KB from the PC -- the
    symbolicator read the wrong image. Clustering on it would present it as a cause."""
    b = boot(
        "BUGCHECK",
        ["BugCheck 0x00000050", "hang: CPU#0 0x80211b55 -> 0014cb55\tXHF_NOEXECUTE+314197\t-"],
    )
    assert flakes.boot_signature(b) == "BUGCHECK 0x50"


def test_hang_without_a_symbol_clusters_on_the_normalized_last_serial_line():
    """Two instances of one wedge differ only in counters; they must be one cluster."""
    mk = lambda seq: boot(  # noqa: E731
        "HANG_TIMEOUT",
        [
            f"timeout: serial SILENT 120s ... last serial: [CENSUS]   WrLpcReply={seq}",
            "0x000007FFB70018CC -> -fffff0008ad8e734\t<no symbol>",
        ],
        hang_kind="wedged",
    )
    assert flakes.boot_signature(mk(1)) == flakes.boot_signature(mk(7))
    assert flakes.boot_signature(mk(1)).startswith("HANG_TIMEOUT wedged | [CENSUS]")


def test_payload_failure_is_named_even_with_no_top_level_classification():
    """115 fleet smoke failures had classification "" -- all payload-stage failures."""
    summ = {
        "verdict": "FAIL",
        "classification": "",
        "boots": [{"ok": True}],
        "stages": [
            {"name": "payload", "status": "fail", "detail": "payload suite (ucrtdiff) FAILED"}
        ],
    }
    assert flakes.run_signature(summ) == "FAIL payload: payload suite (ucrtdiff) FAILED"
    assert flakes.run_signature({"verdict": "OK"}) is None


def test_one_lost_boot_reported_per_test_is_ONE_signature():
    """95 kmtests 'never launched' in one round is one run-level event, not 95 flakes."""
    rec = {
        "shape": "clean-but-ratchet-regression",
        "regressions": [f"T{i}: in baseline (OK) but never launched" for i in range(95)],
    }
    assert flakes.conformance_signatures(rec) == [
        "mass: in baseline (OK) but never launched (N tests)"
    ]


def test_several_asserts_in_one_test_is_one_event_and_drift_is_not_a_second():
    rec = {
        "shape": "clean-but-ratchet-regression",
        "regressions": [
            "RtlRangeList: NEW failing assertion RtlRangeList.c:349",
            "RtlRangeList: NEW failing assertion RtlRangeList.c:357",
            "RtlRangeList: NEW failing assertion RtlRangeList.c:358",
            "RtlRangeList: pass 40 -> 37 (dropped)",
            "ExResource: NEW failing assertion ExResource.c:432",
            "ExResource: pass 900 -> 899 (dropped)",
            "NpfsConnect: pass 679 -> 678 (dropped)",
        ],
    }
    assert flakes.conformance_signatures(rec) == [
        "NpfsConnect: count drift",
        "RtlRangeList: several new failing assertions",
        "assert ExResource.c:432",
    ]
    assert flakes.conformance_signatures({"shape": "clean"}) == []
    assert flakes.conformance_signatures({"shape": "A:bugcheck 0x0000001E"}) == ["BUGCHECK 0x1E"]


# ------------------------------------------------------------------------ grading


def s(head, sigs, sees="all", dirty=None, source="conformance", count=True):
    return {
        "source": source,
        "arch": "i386",
        "head": head,
        "dirty": dirty,
        "ts": 1,
        "sigs": sigs,
        "sees": sees,
        "where": None,
        "count": count,
    }


def one(rows, sig):
    [r] = [r for r in rows if r["sig"] == sig]
    return r


def test_flake_is_present_and_observed_absent_at_one_commit():
    rows = flakes.grade([s("a", ["assert X.c:1"]), s("a", [])])
    assert one(rows, "assert X.c:1")["grade"] == "FLAKE"


def test_a_lost_boot_is_NOT_evidence_that_an_assertion_passed():
    """The NpfsFileInfo trap: a deterministic assertion at a commit where another
    round lost its boot. That round never reached the assertion, so it cannot say
    the assertion passed -- grading it absent turns a real bug into a 'flake'."""
    rows = flakes.grade(
        [
            s("a", ["assert X.c:1"]),
            s("a", ["assert X.c:1"]),
            s("a", ["B:no-launch"], sees="shape"),
        ]
    )
    assert one(rows, "assert X.c:1")["grade"] == "DETERMINISTIC"
    # ...but a lost boot IS a run-level event the complete rounds observed absent.
    assert one(rows, "B:no-launch")["grade"] == "FLAKE"


def test_a_failure_with_another_signature_proves_nothing_about_this_one():
    rows = flakes.grade(
        [
            s("a", ["HANG x"], sees="own", source="smoke"),
            s("a", ["BUGCHECK 0x1E"], sees="own", source="smoke"),
        ]
    )
    assert {r["grade"] for r in rows} == {"UNGRADED"}


def test_dirty_trees_are_never_graded_but_are_counted():
    rows = flakes.grade([s("a", ["assert X.c:1"], dirty=True), s("a", [], dirty=True)])
    r = one(rows, "assert X.c:1")
    assert (r["count"], r["dirty"], r["grade"]) == (0, 1, "UNGRADED")


def test_history_signature_rows_do_not_double_count_runs():
    """A pre-`sig` smoke failure is one ledger row plus one preserved dir; only the
    ledger row is a run."""
    rows = flakes.grade(
        [
            s("a", [], sees="own", source="smoke"),  # the ledger row (sig unknown)
            s("a", ["BUGCHECK 0x50"], sees="own", source="smoke", count=False),  # its evidence
            s("a", [], source="smoke"),  # a green run
        ]
    )
    r = one(rows, "BUGCHECK 0x50")
    assert r["grade"] == "FLAKE" and r["rate"] == 0.5


# ------------------------------------------------------------------- ledger rows


def test_a_failed_smoke_row_names_its_failure_without_the_run_dir(tmp_path, monkeypatch):
    """/tmp is wiped on reboot, so the ledger row itself must carry the signature."""
    monkeypatch.setenv("VIBEOS_ROOT", str(tmp_path))
    monkeypatch.setattr(artifacts, "FARM_DIR", tmp_path / "farm")
    repo = tmp_path / "lane"
    repo.mkdir()
    subprocess.run(["git", "init", "-q", str(repo)], check=True)
    run = Run("smoke", "arm64", ts=1700000001)
    run.add(Stage("boot", status="fail", detail="BUGCHECK"))
    run.classification = "BUGCHECK"
    run.extra.update(
        {
            "runs": 1,
            "passes": 0,
            "boots": [boot("BUGCHECK", ["BugCheck 0x000000E3 (after boot marker)"])],
        }
    )
    run.finalize(repo)
    [row] = [json.loads(ln) for ln in (tmp_path / "vibeos-ledger.jsonl").read_text().splitlines()]
    assert row["sig"] == "BUGCHECK 0xE3 post-marker"
    assert row["run_id"] == run.dir.name
    assert row["dirty"] is False
    assert (row["boots"], row["passes"]) == (1, 0)


def test_collect_reads_signed_ledger_rows(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_ROOT", str(tmp_path))
    monkeypatch.setattr(flakes, "PRESERVED", tmp_path / "none")
    monkeypatch.setattr(flakes, "CONF_ROUNDS", tmp_path / "none.jsonl")
    rows = [
        {
            "ts": 2e9,
            "cmd": "smoke",
            "arch": "i386",
            "verdict": "FAIL",
            "head": "a",
            "classification": "BUGCHECK",
            "sig": "BUGCHECK 0x1E",
            "run_id": "r1",
            "dirty": False,
        },
        {
            "ts": 2e9,
            "cmd": "smoke",
            "arch": "i386",
            "verdict": "OK",
            "head": "a",
            "classification": "",
            "run_id": "r2",
            "dirty": False,
        },
        {
            "ts": 2e9,
            "cmd": "smoke",
            "arch": "i386",
            "verdict": "FAIL",
            "head": "a",
            "classification": "STALE_MEDIA",
            "run_id": "r3",
        },
        {
            "ts": 2e9,
            "cmd": "conformance",
            "arch": "i386",
            "verdict": "FAIL",
            "head": "a",
            "classification": "clean-but-ratchet-regression",
            "sigs": ["assert X.c:1"],
        },
    ]
    (tmp_path / "vibeos-ledger.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
    graded = flakes.grade(flakes.collect(tmp_path, since=0))
    assert one(graded, "BUGCHECK 0x1E")["grade"] == "FLAKE"  # STALE_MEDIA is not a boot
    assert one(graded, "assert X.c:1")["grade"] == "UNGRADED"


# ----------------------------------------------------------- lost boots, split


def test_a_round_with_no_guest_serial_is_not_a_lost_boot():
    """F3: QEMU never connected, so the serial holds only the harness header. That is
    the host, not the OS -- it must not share B:no-launch with kernels that died."""
    from .conformance import round_shape

    header = "# vibeos smoke live serial -- amd64-uefi -- pid(launcher)=1\n# tail -f x\n"
    assert round_shape(None, [], None, [], header) == "L:no-guest-serial"
    assert round_shape(None, [], None, [], header + "BdsDxe: loading Boot0001\n") == "B:no-launch"
    # a bugcheck is still named first, whatever else the round shows
    assert round_shape("0x1E", [], None, [], header) == "A:bugcheck 0x1E"
    assert flakes.conformance_signatures({"shape": "L:no-guest-serial"}) == ["L:no-guest-serial"]


def test_every_gated_boot_path_launches_through_the_retry():
    """The retry lived in cli/smoke.py, so the pytest `vm` fixture (conformance, the
    payload suites, the net suite) kept a bare `vm.start()` and lost rounds to the
    exact socket flake smoke had already fixed. Pin both call sites to the shared one."""
    import inspect
    from pathlib import Path

    from .. import conftest
    from . import smoke

    fixture = Path(conftest.__file__).read_text()
    body = fixture[fixture.index("def vm(") : fixture.index("\ndef ", fixture.index("def vm(") + 1)]
    assert "launch_with_retry(" in body
    assert "vm.start()" not in body
    assert "launch_with_retry(" in inspect.getsource(smoke._boot_vm)


def test_no_build_refuses_media_without_the_sweep(tmp_path):
    """A plain build's media boots to a desktop with no sweep; --no-build on it drained
    30 min per arch and banked fake lost boots. The payload stamp answers first."""
    import json as _json

    from .conformance import SUITE, media_payload_gap

    assert "missing" in (media_payload_gap(tmp_path) or "")
    (tmp_path / "payload-selection.json").write_text(_json.dumps({"payloads": []}))
    assert "not" in (media_payload_gap(tmp_path) or "")
    (tmp_path / "payload-selection.json").write_text(_json.dumps({"payloads": [SUITE]}))
    assert media_payload_gap(tmp_path) is None
