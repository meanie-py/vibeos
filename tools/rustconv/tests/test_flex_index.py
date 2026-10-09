"""flex_index: C flexible arrays indexed without a Rust bounds check, as the C does."""

from rustconv import flex_index as fi

FIELDS = frozenset({"MountPoints", "PartitionEntry"})


def test_reads_and_writes_become_unchecked_pointer_arithmetic():
    src = (
        "let n = (*data).MountPoints[(*data).Size as usize].SymbolicLinkNameLength;\n"
        "(*(*DiskEntry).LayoutBuffer).PartitionEntry[i as usize].RewritePartition = 1;\n"
        "PartitionInfo = &mut (*(*DiskEntry).LayoutBuffer).PartitionEntry[Index as usize];\n"
        "let first = (*data).MountPoints[0].X;\n"
        "// (*data).MountPoints[i]\n"
    )
    out, n = fi.rewrite(src, FIELDS)
    assert n == 3
    assert (
        "(*core::ptr::addr_of!((*data).MountPoints[0]).add((*data).Size as usize)).Symbolic" in out
    )
    assert (
        "(*core::ptr::addr_of_mut!((*(*DiskEntry).LayoutBuffer).PartitionEntry[0])"
        ".add(i as usize)).RewritePartition = 1;" in out
    )
    assert "&mut (*core::ptr::addr_of_mut!(" in out
    assert "(*data).MountPoints[0].X" in out  # [0] cannot panic
    assert "// (*data).MountPoints[i]" in out  # comments untouched


def test_the_verified_form_only_compiles_for_a_one_element_array_and_reverts():
    out, n = fi.rewrite("x = (*p).PartitionEntry[k];\n", FIELDS, verified=True)
    assert n == 1
    assert "let flex: *const [_; 1] = core::ptr::addr_of!((*p).PartitionEntry);" in out
    back = fi._VERIFIED.sub(lambda m: f"{m.group('arr')}[{fi._unwrap(m.group('idx'))}]", out)
    assert back == "x = (*p).PartitionEntry[k];\n"


def test_field_shapes_split_flexible_from_ambiguous(tmp_path):
    (tmp_path / "isl.rs").write_text(
        "pub struct _A {\n    pub Count: u32,\n    pub Items: [ITEM; 1usize],\n}\n"
        "pub struct _B {\n    pub Name: [u16; 1usize],\n    pub Len: u32,\n}\n"
        "pub struct _C {\n    pub Size: u32,\n    pub Table: [u8; 1usize],\n}\n"
        "pub struct _D {\n    pub Table: [u8; 4096usize],\n}\n"
    )
    flex, ambiguous = fi.field_shapes.__wrapped__(tmp_path)
    assert flex == {"Items"}  # Name is not the last field
    assert ambiguous == {"Table"}


def test_a_place_broken_across_lines_is_still_rewritten():
    # pciidex's resource walk, as rustfmt wrote it: the first --ata-stack Rust panic.
    from rustconv import flex_index as fi

    src = (
        "let d = &mut (*R).List[0]\n            .PartialResourceList\n"
        "            .PartialDescriptors[i as usize];\n"
    )
    out, n = fi.rewrite(src, frozenset({"PartialDescriptors"}))
    assert n == 1
    assert "core::ptr::addr_of_mut!((*R).List[0]" in out and ".add(i as usize)" in out
