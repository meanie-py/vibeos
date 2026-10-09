from rustconv.pointer_depth import c_declarations
from rustconv.union_slot import scan

C = """UCHAR PataReadStatus(PCHANNEL_DATA_PATA ChanData)
{
    return ATA_READ(ChanData->Regs.Status, ChanData, MRES_TF);
}"""


def _decls(body: str, name: str):
    return {name: c_declarations(name, [], body, set())}


def test_a_member_the_c_never_names_is_reported():
    rust = (
        'unsafe extern "C" fn PataReadStatus(ChanData: PCHANNEL_DATA_PATA) -> UCHAR {\n'
        "    return ATA_READ!((*ChanData).Regs.__bindgen_anon_1.Error, ChanData, MRES_TF);\n"
        "}\n"
    )
    hits = scan(rust, _decls(C, "PataReadStatus"))
    assert [(h.fn, h.member, h.line) for h in hits] == [("PataReadStatus", "Error", 2)]


def test_the_named_member_passes_and_unknown_functions_are_skipped():
    good = (
        'unsafe extern "C" fn PataReadStatus(ChanData: PCHANNEL_DATA_PATA) -> UCHAR {\n'
        "    return ATA_READ!((*ChanData).Regs.__bindgen_anon_5.Status, ChanData, MRES_TF);\n"
        "}\n"
        "fn Other() { x.__bindgen_anon_1.Error = 0; }\n"
    )
    assert scan(good, _decls(C, "PataReadStatus")) == []


def test_a_member_written_more_often_than_the_c_writes_it():
    c = """VOID Init(PIDE_REGISTERS Registers)
{
    Registers->Error = Base + 1;
    Registers->Status = Base + 7;
}"""
    rust = (
        "unsafe fn Init(Registers: PIDE_REGISTERS) {\n"
        "    (*Registers).__bindgen_anon_1.Error = Base + 1;\n"
        "    (*Registers).__bindgen_anon_1.Error = Base + 7;\n"
        "}\n"
    )
    hits = scan(rust, _decls(c, "Init"))
    assert [(h.member, h.line) for h in hits] == [("Error", 3)]


def test_a_write_copied_by_goto_elimination_is_not_a_reslot():
    c = """NTSTATUS Pnp(PIRP Irp)
{
    if (x) goto CompleteIrp;
    Irp->IoStatus.Status = Status;
    return Status;
CompleteIrp:
    Irp->IoStatus.Status = Status;
    return Status;
}"""
    rust = (
        "unsafe fn Pnp(Irp: PIRP) -> NTSTATUS {\n"
        "    if x { (*Irp).IoStatus.__bindgen_anon_1.Status = Status; return Status; }\n"
        "    if y { (*Irp).IoStatus.__bindgen_anon_1.Status = Status; return Status; }\n"
        "    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;\n"
        "    return Status;\n"
        "}\n"
    )
    assert scan(rust, _decls(c, "Pnp")) == []
