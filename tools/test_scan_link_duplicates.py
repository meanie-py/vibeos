"""scan_link_duplicates: parse lld's duplicate-symbol warnings as rustc relays them."""

import json

import scan_link_duplicates as sld

_STDERR = """linker stderr: clang: argument unused during compilation: '-no-pie'
ld.lld: duplicate symbol: ProbeForWriteHandle
>>> defined at /vibeos/ntoskrnl/io/iomgr/io_rust_shims.c:84
>>>            /tmp/vibeos/sdk/lib/rust/target/x/out/objs/io_rust_shims.o
>>> defined at ntoskrnl-rust/src/common_inlines.rs:112
>>>            libkernel_rust.a(ntoskrnl_rust.cgu.041.rcgu.o)

ld.lld: duplicate symbol: _KeRaiseIrqlToSynchLevel@0
>>> defined at /vibeos/ntoskrnl/ke/ke_rust_shims.c:130
>>>            /tmp/vibeos/x/native-4f86.o
>>> defined at hal.dll

ld.lld: duplicate symbol: __rustc::rust_begin_unwind
>>> defined at ntoskrnl-cargo/src/main.rs:38
>>>            ntoskrnl.cgu.0.rcgu.o
>>> defined at pseh-rust/src/lib.rs:33
>>>            libpseh_rust.a(pseh_rust.cgu.0.rcgu.o)
"""


def test_parse_pairs_each_definer_with_its_object():
    blocks = dict(sld.parse(_STDERR))
    a, b = blocks["ProbeForWriteHandle"]
    assert a.where.endswith("io_rust_shims.c:84") and a.obj.endswith("io_rust_shims.o")
    assert b.where == "ntoskrnl-rust/src/common_inlines.rs:112"
    assert b.obj.startswith("libkernel_rust.a(")
    # an import-library definer has no object line
    _, d = blocks["_KeRaiseIrqlToSynchLevel@0"]
    assert d.where == d.obj == "hal.dll"


def test_structural_runtime_is_not_a_finding():
    assert sld.structural("__rustc::rust_begin_unwind")
    assert sld.structural("_rust_eh_personality")
    assert sld.structural("___rust_alloc")
    assert not sld.structural("ProbeForWriteHandle")
    assert not sld.structural("_RtlFillMemoryUlong@12")


def test_findings_reads_the_cached_cargo_diagnostic(tmp_path):
    release = "sdk/lib/rust/target/nt-cargo-amd64-cargo-link/x86_64-pc-windows-gnu/release"
    fp = tmp_path / release / ".fingerprint/nt-1"
    fp.mkdir(parents=True)
    (fp / "bin-nt").write_text("")
    (fp / "output-bin-nt").write_text(json.dumps({"message": _STDERR}) + "\n")
    found, links = sld.findings("amd64", tmp_path)
    assert links == 1
    # The KeRaiseIrqlToSynchLevel definition in the real ke_rust_shims.c carries
    # `nolint: duplicate-symbol`, so only the probe pair is a finding.
    assert [f.symbol for f in found] == ["ProbeForWriteHandle"]
