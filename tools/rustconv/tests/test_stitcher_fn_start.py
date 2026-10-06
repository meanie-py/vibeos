"""An attribute INSIDE a body is not where the function starts."""

from pathlib import Path

from rustconv.callgraph import FunctionDep
from rustconv.config import TARGETS
from rustconv.resolver import ResolveResult
from rustconv.scanner import ScanResult
from rustconv.stitcher import stitch_file

BODY = """unsafe extern "C" fn UpdateDiskLayout(d: u32) {
    let x = d;
    #[cfg(feature = "DUMP_PARTITION_TABLE")]
    DumpPartitionTable(x);
}"""


def test_non_pub_extern_fn_with_an_inner_attribute_stays_whole():
    functions = {"UpdateDiskLayout": BODY, "Later": "pub unsafe fn Later() {}"}
    deps = [
        FunctionDep(name="UpdateDiskLayout", line_start=900),
        FunctionDep(name="Later", line_start=10),
    ]
    out = stitch_file(
        functions, deps, ScanResult(path=Path("x.c")), ResolveResult(), target=TARGETS["ntoskrnl"]
    )
    start = out.index('unsafe extern "C" fn UpdateDiskLayout')
    # the body is contiguous: its inner attribute and closing brace follow it directly
    assert out[start:].startswith(BODY)
    # and source order still holds (Later is at C line 10)
    assert out.index("fn Later") < start
