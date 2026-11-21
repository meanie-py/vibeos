"""wined3d directx.c: 51K of a 52K preamble was c2rust's exact data, and the reply truncated."""

from rustconv.preamble_trim import omit_c2rust_data

SRC = """struct gpu_description { WORD vendor; const char *description; };
static const struct gpu_description gpu_description_table[] =
{
    {0x10de, "NVIDIA {GeForce}; 256"},
    {0x1002, "AMD"},
};
static HANDLE notification_thread;
static const GUID IID_X = {0x1, 0x2, 0x3, {0}};
"""


def test_c2rust_data_leaves_the_prompt_and_everything_else_stays():
    got, omitted = omit_c2rust_data(SRC, {"gpu_description_table", "IID_X", "absent"})
    assert omitted == ["IID_X", "gpu_description_table"]
    assert "0x10de" not in got and "0x1, 0x2" not in got
    assert "struct gpu_description { WORD vendor;" in got
    assert "static HANDLE notification_thread;" in got
    assert "/* gpu_description_table: defined exactly by c2rust" in got


def test_a_declaration_without_an_initialiser_is_not_a_definition_to_drop():
    got, omitted = omit_c2rust_data(SRC, {"notification_thread"})
    assert omitted == [] and got == SRC
