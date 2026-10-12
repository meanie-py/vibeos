"""tools/link_guard.py: which duplicate symbols fail the link.

The stderr samples are verbatim lld output from the 2026-10-04 census.
"""
# ruff: noqa: E501  (the lld samples are verbatim)

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import link_guard as g

CDROM = """\
ld.lld: warning: duplicate symbol: DeviceEvtIoInCallerContext
>>> defined at /drivers/storage/class/cdrom/rust_dbgprint_shim.c:506
>>>            drivers/storage/class/cdrom/CMakeFiles/cdrom.dir/rust_dbgprint_shim.c.obj
>>> defined at cdrom-rust/src/cdrom.rs:1566
>>>            libcdrom_rust.a(cdrom_rust-fe749c8c2af7958f.cdrom_rust.a2994185a285f8da-cgu.02.rcgu.o)
"""


def verdicts(stderr: str) -> list[str]:
    return [g.judge(*t).verdict for t in g.parse(stderr)]


def test_parse_keeps_first_definition():
    [(sym, kept, dropped)] = g.parse(CDROM)
    assert sym == "DeviceEvtIoInCallerContext"
    assert "rust_dbgprint_shim.c.obj" in kept
    assert "libcdrom_rust.a" in dropped


def test_c_over_rust_fails():
    assert verdicts(CDROM) == ["error:two local definitions, kept by link order"]


def test_toolchain_runtime_twice_is_fine():
    stderr = """\
ld.lld: warning: duplicate symbol: __udivti3
>>> defined at library\\compiler-builtins\\compiler-builtins\\src/macros.rs:478
>>>            libmsvcrt_crt_cargo.a(compiler_builtins-87953880d1763386.compiler_builtins.ace4d55401a4cf31-cgu.145.rcgu.o)
>>> defined at libclang_rt.builtins-x86_64.a(udivti3.c.obj)
ld.lld: warning: duplicate symbol: ___udivdi3
>>> defined at .\\lib/builtins\\i386/udivdi3.S:26
>>>            libcdrom_rust.a(b010cb07759ed85a-udivdi3.o)
>>> defined at libclang_rt.builtins-i386.a(udivdi3.S.obj)
"""
    assert verdicts(stderr) == ["ok:toolchain runtime twice"] * 2


def test_rust_panic_runtime_is_fine():
    stderr = """\
ld.lld: warning: duplicate symbol: __rustc::rust_begin_unwind
>>> defined at lsa-stub/src/lib.rs:17
>>>            liblsa_stub.a(lsa_stub-eb29d5b01a24d5d0.lsa_stub.f5990ff3841e7454-cgu.0.rcgu.o)
>>> defined at lsasrv-rust/src/lib.rs:58
>>>            liblsasrv_rust.a(lsasrv_rust-a7506266d3c25ff1.lsasrv_rust.5f9538ccbc50f085-cgu.16.rcgu.o)
"""
    assert verdicts(stderr) == ["ok:Rust panic runtime"]


def test_stub_kept_over_real_unwinder_fails():
    stderr = """\
ld.lld: warning: duplicate symbol: _Unwind_Resume
>>> defined at panic-common/src/lib.rs:146
>>>            libconutils_rust.a(panic_common-6e1eed3e6cd48ad3.panic_common.ec2e1bb8b65daef6-cgu.0.rcgu.o)
>>> defined at libunwind.a(Unwind-seh.cpp.obj)
"""
    assert verdicts(stderr) == ["error:a local copy was kept over the toolchain runtime"]


def test_imports():
    local_kept = """\
ld.lld: warning: duplicate symbol: SHLockShared
>>> defined at dll/win32/shlwapi/CMakeFiles/shlwapi.dir/ordinal.c.obj
>>> defined at libshell32_delayed.a(libshell32_delayed.delay.o)
ld.lld: warning: duplicate symbol: RtlAssert
>>> defined at usbstor-rust/src/glue.rs:120
>>>            libusbstor_rust.a(usbstor_rust-9cf353b62bfb5b8a.usbstor_rust.d06e47a54a40a025-cgu.06.rcgu.o)
>>> defined at ntoskrnl.exe
ld.lld: warning: duplicate symbol: GetErrorInfo
>>> defined at libole32_delayed.a(libole32_delayed.delay.o)
>>> defined at liboleaut32_delayed.a(liboleaut32_delayed.delay.o)
"""
    assert verdicts(local_kept) == ["ok:import"] * 3
    import_kept = """\
ld.lld: warning: duplicate symbol: SHLockShared
>>> defined at libshell32_delayed.a(libshell32_delayed.delay.o)
>>> defined at dll/win32/shlwapi/CMakeFiles/shlwapi.dir/ordinal.c.obj
"""
    assert verdicts(import_kept) == ["error:an import was kept over a local definition"]


def test_other_warnings_ignored():
    assert g.parse("ld.lld: warning: .edata=.rdata: section merging\n") == []


def test_output_of():
    assert g.output_of(["-m", "i386pep", "-o", "x/rapps.exe", "a.o"]) == "x/rapps.exe"
    assert g.output_of(["-ox.dll"]) == "x.dll"


def test_crt_mem_over_compiler_builtins_is_fine():
    stderr = """\
ld.lld: warning: duplicate symbol: _memcpy
>>> defined at /vibeos/sdk/lib/crt/mem/memcpy.c:9
>>>            /tmp/vibeos/sdk/lib/rust/target/ntoskrnl-cargo-i386-cargo-link/i686-kernel/release/build/x/out/memcpy.o
>>> defined at src/macros.rs:415
>>>            libcompiler_builtins-a041ab404c42ddb5.rlib(compiler_builtins-a041ab404c42ddb5.compiler_builtins.644fedaf30b7597b-cgu.136.rcgu.o)
"""
    assert verdicts(stderr) == ["ok:the CRT's own mem* over compiler_builtins' copy"]


def test_two_varargs_shims_share_their_pullers():
    # autochk.exe links its own rust_varargs_shim.c and vfatlib's (a static library).
    stderr = """\
ld.lld: warning: duplicate symbol: _vibeos_va_arg_i32
>>> defined at /base/system/autochk/rust_varargs_shim.c:12
>>>            base/system/autochk/CMakeFiles/autochk.dir/rust_varargs_shim.c.obj
>>> defined at /sdk/lib/fslib/vfatlib/rust_varargs_shim.c:12
>>>            sdk/lib/fslib/vfatlib/CMakeFiles/vfatlib.dir/rust_varargs_shim.c.obj
"""
    assert verdicts(stderr) == ["ok:rustconv varargs puller (one generated body)"]
    # Not a shim on one side: still a real collision.
    other = stderr.replace("/sdk/lib/fslib/vfatlib/rust_varargs_shim.c:12", "/x/handwritten.c:3")
    assert verdicts(
        other.replace("vfatlib.dir/rust_varargs_shim.c.obj", "x.dir/handwritten.c.obj")
    ) == ["error:two local definitions, kept by link order"]
