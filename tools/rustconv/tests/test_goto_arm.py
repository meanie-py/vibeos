from rustconv.goto_arm import c_goto_arms, scan
from rustconv.pointer_depth import c_declarations

C = """NTSTATUS PciIdeXFdoDispatchPnp(PVOID FdoExtension, PIRP Irp)
{
    switch (IoStack->MinorFunction)
    {
        case IRP_MN_START_DEVICE:
        {
            if (!IoForwardIrpSynchronously(Lower, Irp))
            {
                Status = STATUS_UNSUCCESSFUL;
                goto CompleteIrp;
            }
            Status = PciIdeXFdoStartDevice(FdoExtension, Irp);
            goto CompleteIrp;
        }
        case IRP_MN_STOP_DEVICE:
            if (x)
                goto CompleteIrp;
            break;
        case IRP_MN_QUERY_ID:
            Status = Q();
            goto Forward;
        default:
            goto CompleteIrp;
    }
Forward:
    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver(Lower, Irp);

CompleteIrp:
    Irp->IoStatus.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return Status;
}"""


def _decls():
    return {"PciIdeXFdoDispatchPnp": c_declarations("PciIdeXFdoDispatchPnp", [], C, set())}


def test_c_side_keeps_only_unconditional_tail_gotos_past_the_switch():
    # STOP's goto is conditional; QUERY_ID jumps to the statement right after the switch;
    # `default` has no case name to find in the Rust.
    assert c_goto_arms(C) == (("IRP_MN_START_DEVICE", "CompleteIrp"),)


def _rust(arm: str) -> str:
    return (
        'pub unsafe extern "C" fn PciIdeXFdoDispatchPnp(Fdo: PVOID, Irp: PIRP) -> NTSTATUS {\n'
        "    match (*IoStack).MinorFunction {\n"
        f"        IRP_MN_START_DEVICE => {{\n{arm}\n        }}\n"
        "        _ => {}\n"
        "    }\n"
        "    IoSkipCurrentIrpStackLocation(Irp);\n"
        "    return IoCallDriver(Lower, Irp);\n"
        "}\n"
    )


def test_an_arm_that_falls_out_is_reported():
    hits = scan(_rust("            Status = PciIdeXFdoStartDevice(FdoExtension, Irp);"), _decls())
    assert [(h.fn, h.case, h.label, h.line) for h in hits] == [
        ("PciIdeXFdoDispatchPnp", "IRP_MN_START_DEVICE", "CompleteIrp", 3)
    ]


def test_arms_that_leave_pass():
    for arm in (
        "            Status = S();\n            return Status;",
        "            'CompleteIrp: {\n                IoCompleteRequest(Irp, 0);\n"
        "                return Status;\n            }",
        "            if x { return A; } else if y { return B; } else { break 'done; }",
    ):
        assert scan(_rust(arm), _decls()) == [], arm


def test_an_if_without_else_still_falls_out():
    assert len(scan(_rust("            if x { return A; }"), _decls())) == 1


def test_a_goto_flag_tested_after_the_match_is_the_jump():
    # servman's MainWndProc: `goto_default = true;` for `goto HandleDefaultMessage;`, with
    # DefWindowProc under `if goto_default { .. }` after the match.
    def rust(flag: str, tested: bool) -> str:
        return (
            'pub unsafe extern "C" fn PciIdeXFdoDispatchPnp(Fdo: PVOID, Irp: PIRP) -> NTSTATUS {\n'
            f"    let mut {flag}: bool = false;\n"
            "    match (*IoStack).MinorFunction {\n"
            "        IRP_MN_START_DEVICE => {\n"
            "            Status = PciIdeXFdoStartDevice(Fdo, Irp);\n"
            f"            {flag} = true;\n"
            "        }\n"
            "        _ => {}\n"
            "    }\n"
            + (f"    if {flag} {{\n        return Complete(Irp, Status);\n    }}\n" if tested else "")
            + "    return IoCallDriver(Lower, Irp);\n}\n"
        )

    assert scan(rust("goto_complete", True), _decls()) == []
    assert scan(rust("completeirp", True), _decls()) == []
    # A flag nobody tests, or one whose name says nothing about the jump, is still a fall.
    assert len(scan(rust("goto_complete", False), _decls())) == 1
    assert len(scan(rust("done", True), _decls())) == 1
