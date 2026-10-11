import shutil
import subprocess

import pytest

from rustconv.cpp_bridge import BridgeError, generate_bridge, source_macros
from rustconv.scanner import scan_file


SOURCE = """#include "fixture.hpp"
class Counter {
public:
    int m_Value;
    Counter() : m_Value(4) {}
    int Add(int amount);
    int Get();
    int Copy(Parent *other);
};
int Counter::Add(int amount) { m_Value += amount; return m_Value; }
int Counter::Get() { return m_Value; }
int Counter::Copy(Parent *other) { return Add(other->Get()); }
"""


def make(tmp_path, source=SOURCE):
    path = tmp_path / "counter.cpp"
    path.write_text(source)
    return generate_bridge(
        source,
        scan_file(path),
        "Counter",
        ["Add", "Get", "Copy"],
        header_name="bridge.h",
        includes="typedef struct Parent Parent;",
    )


def test_compiler_bridge_matches_original_state_and_virtual_calls(tmp_path):
    cc, cxx = shutil.which("cc"), shutil.which("c++")
    if not cc or not cxx:
        pytest.skip("native C and C++ compilers required")
    result = make(tmp_path)
    (tmp_path / "fixture.hpp").write_text("struct Parent { virtual int Get() = 0; };\n")
    (tmp_path / "debug.h").write_text("")
    (tmp_path / "bridge.h").write_text(result.header)
    (tmp_path / "reference.c").write_text(result.reference)
    harness = """
#include <cstdio>
struct Other : Parent { int Get() override { return 7; } };
int main() { Counter c; Other other; for (int i = -10; i < 10; ++i) {
std::printf("%d %d %d\\n", c.Add(i), c.Copy(&other), c.Get()); } }
"""
    (tmp_path / "original.cpp").write_text(SOURCE + harness)
    (tmp_path / "bridge.cpp").write_text(result.bridge + harness)
    commands = [
        [cc, "-I.", "-c", "reference.c", "-o", "reference.o"],
        [cxx, "-std=c++11", "original.cpp", "-o", "original"],
        [cxx, "-std=c++11", "bridge.cpp", "reference.o", "-o", "bridged"],
    ]
    for command in commands:
        completed = subprocess.run(command, cwd=tmp_path, capture_output=True, text=True)
        assert completed.returncode == 0, completed.stderr
    original = subprocess.check_output([str(tmp_path / "original")])
    bridged = subprocess.check_output([str(tmp_path / "bridged")])
    assert original == bridged
    assert "Counter() : m_Value(4)" in result.bridge
    assert "int *m_Value;" in result.header
    assert "(*context->m_Value) += amount" in result.reference


def test_complex_receiver_is_refused_before_any_worker_purchase(tmp_path):
    with pytest.raises(BridgeError, match="complex member call"):
        make(tmp_path, SOURCE.replace("other->Get()", "(other + 1)->Get()"))


def test_macro_definition_is_copied_verbatim_and_ambiguity_refused():
    source = "#define PC_ASSERT(x) \\\n ((x) ? 1 : fail())\n"
    assert source_macros(source, ["PC_ASSERT"]) == source
    with pytest.raises(BridgeError, match="expected one definition"):
        source_macros(source + source, ["PC_ASSERT"])


def test_conditional_macro_cannot_escape_its_guard():
    with pytest.raises(BridgeError, match="conditional definition"):
        source_macros("#ifdef DBG\n#define PC_ASSERT(x) check(x)\n#endif\n", ["PC_ASSERT"])
    guarded = "#ifndef FIXTURE_H\n#define FIXTURE_H\n#define PC_ASSERT(x) check(x)\n#endif\n"
    assert source_macros(guarded, ["PC_ASSERT"]) == "#define PC_ASSERT(x) check(x)\n"


def test_owner_cannot_treat_an_ordinary_c_file_as_a_validated_bridge(tmp_path):
    from rustconv.cpp_bridge import validate_reference

    source = tmp_path / "fake_reference.c"
    source.write_text("void unrelated(void) {}")
    with pytest.raises(BridgeError, match="compiler provenance"):
        validate_reference(source, required=True)
