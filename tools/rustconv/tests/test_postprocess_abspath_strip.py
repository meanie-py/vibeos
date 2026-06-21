"""Determinism guard test: strip_embedded_abspaths must scrub the host/worktree
root that c2rust bakes into translated __FILE__ string literals, leaving a
repo-relative (deterministic, portable) path. See deterministic-builds-plan.md
rung 1 + the 1307-path committed sweep this prevents from recurring.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.config import REACTOS_ROOT
from rustconv.postprocess import postprocess, strip_embedded_abspaths


def test_strips_repo_root_to_relative():
    root = str(REACTOS_ROOT)
    code = f'    {chr(98)}"{root}/dll/win32/kernel32/winnls/string/nls.c\\0".as_ptr()'
    out = strip_embedded_abspaths(code)
    assert root not in out, f"repo root not stripped: {out}"
    assert 'b"dll/win32/kernel32/winnls/string/nls.c\\0"' in out, out


def test_noop_on_already_relative():
    code = '    b"dll/win32/kernel32/winnls/string/nls.c\\0".as_ptr()'
    assert strip_embedded_abspaths(code) == code


def test_wired_into_postprocess():
    # the full postprocess pass applies the strip (first transform)
    root = str(REACTOS_ROOT)
    code = f'pub fn f() {{ let _ = {chr(98)}"{root}/ntoskrnl/ke/foo.c\\0"; }}'
    out = postprocess(code)
    assert root not in out, out
