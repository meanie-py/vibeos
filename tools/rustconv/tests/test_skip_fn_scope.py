"""`--skip-fn` is scoped to the file that HAS the name, not to every file in the worklist.

THE BUG THIS PINS. A function name belongs to exactly one translation unit. The original
per-file check errored on any file that did not declare the requested name and `continue`d,
which on a `translate <directory>` run meant every OTHER file was skipped entirely — unbought,
unassembled, and reported only as one stderr line among eleven identical ones. Asking to drop
one BODY silently dropped eleven FILES, and the run exited 0.

The typo case still has to be loud, though, and it is the inverse of the flag's guarantee: the
caller believes a body was left to them to hand-write, when in fact it was translated and
shipped. That check can only be made once the whole worklist has been seen, so it moved out of
the loop and into the audit report.
"""

from __future__ import annotations

from rustconv.__main__ import skip_fn_for_file, unmatched_skip_fn


def test_a_name_this_file_does_not_have_is_not_applicable_here():
    """The regression: hardprof.c owns the name, so advanced.c must translate normally."""
    assert skip_fn_for_file(["HardwareProfileProperties"], ["OnOK", "AdvancedPageProc"]) == []


def test_a_name_this_file_has_is_dropped():
    assert skip_fn_for_file(["OnInitDialog"], ["OnInitDialog", "LicenceDlgProc"]) == [
        "OnInitDialog"
    ]


def test_several_names_across_several_files_each_land_on_their_own():
    """The case the old behaviour could not express at all on a directory run."""
    requested = ["HardwareProfileProperties", "OnInitDialog"]
    assert skip_fn_for_file(requested, ["UpdateButtons", "HardwareProfileProperties"]) == [
        "HardwareProfileProperties"
    ]
    assert skip_fn_for_file(requested, ["OnInitDialog", "LicenceDlgProc"]) == ["OnInitDialog"]


def test_a_name_no_file_carried_is_reported():
    """A misspelled name means a body the caller thinks they own was actually SHIPPED."""
    assert unmatched_skip_fn(["OnInitDialog", "Typo"], ["OnInitDialog"]) == ["Typo"]


def test_every_name_matched_reports_nothing():
    assert unmatched_skip_fn(["A", "B"], ["B", "A"]) == []


def test_no_request_is_never_a_finding():
    assert skip_fn_for_file([], ["A", "B"]) == []
    assert unmatched_skip_fn([], []) == []
