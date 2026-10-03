"""verify_ffi's flexible-array rule must not fire on a fixed array that shares the name."""

from rustconv.verify_ffi import flexible_facts


def test_key_value_data_index_is_flagged():
    text = "let b = (*info).Data[3];\n"
    # Reported once per struct entry naming `Data` (KEY_VALUE_FULL_ and _PARTIAL_).
    assert [f["type"] for f in flexible_facts("x.rs", text)] == ["FLEX_ARRAY", "FLEX_ARRAY"]


def test_device_private_data_is_fixed_not_flexible():
    # CM_PARTIAL_RESOURCE_DESCRIPTOR.u.DevicePrivate.Data is ULONG[3].
    text = "DrainPartial = (*Partial).u.DevicePrivate.Data[1];\n"
    assert flexible_facts("x.rs", text) == []


def test_flexible_use_on_a_line_with_device_private_still_flagged():
    text = "x = (*p).u.DevicePrivate.Data[1] + (*info).Data[2];\n"
    assert len(flexible_facts("x.rs", text)) == 2  # one hit per KEY_VALUE_* struct entry
