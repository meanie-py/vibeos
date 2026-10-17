"""Unit tests for tools/gen_rpc_guards.py (widl client header -> SEH-guarded wrappers)."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_rpc_guards as g

HEADER = """
/* client prototype */
UINT __cdecl remote_GetProperty(
    MSIHANDLE hinst,
    LPCWSTR property,
    LPWSTR *value,
    DWORD *size);

/* server prototype */
UINT __cdecl s_remote_GetProperty(
    MSIHANDLE hinst,
    LPCWSTR property,
    LPWSTR *value,
    DWORD *size);

void __cdecl ept_insert(
    handle_t h,
    unsigned32 num_ents,
    ept_entry_t entries[],
    boolean32 replace,
    error_status_t *status);

LANGID __cdecl remote_GetLanguage(
    MSIHANDLE hinst);

DWORD __cdecl PNP_GetVersion(void);
"""


def test_parses_client_prototypes_and_decays_arrays():
    protos = {p.name: p for p in g.parse_header(HEADER)}
    assert set(protos) == {
        "remote_GetProperty",
        "s_remote_GetProperty",
        "ept_insert",
        "remote_GetLanguage",
        "PNP_GetVersion",
    }
    ins = protos["ept_insert"]
    assert ins.ret == "void"
    assert [(p.type, p.name) for p in ins.params][2] == ("ept_entry_t *", "entries")
    assert protos["PNP_GetVersion"].params == []


def test_wrappers_store_ret_only_for_non_void():
    h, c = g.render(g.parse_header(HEADER), "x_c.h", ["<string.h>"], "x_guard.h")
    assert (
        "BOOL remote_GetProperty_guarded(RPC_GUARD_FILTER guard_filter, DWORD *guard_code, "
        "UINT *guard_ret, MSIHANDLE hinst, LPCWSTR property, LPWSTR *value, DWORD *size);"
    ) in h
    assert (
        "BOOL ept_insert_guarded(RPC_GUARD_FILTER guard_filter, DWORD *guard_code, handle_t h, "
        "unsigned32 num_ents, ept_entry_t *entries, boolean32 replace, "
        "error_status_t *status);"
    ) in h
    assert "a->guard_ret = remote_GetLanguage(a->hinst);" in c
    assert "    ept_insert(a->h, a->num_ents, a->entries, a->replace, a->status);" in c
    # a no-argument stub still gets a non-empty args struct (an empty struct is not C)
    assert "struct PNP_GetVersion_args { DWORD guard_ret; };" in c
    assert '#include "x_c.h"' in c and '#include "x_guard.h"' in c
    assert "RtlSehTryExcept(" in h  # declared for the module's own non-RPC guards too


def test_rejects_a_header_without_client_stubs():
    with pytest.raises(ValueError):
        g.parse_header("int foo(void);")


def test_rejects_function_pointer_parameters():
    with pytest.raises(ValueError):
        g.parse_header("void __cdecl f(\n    void (*cb)(int));")
