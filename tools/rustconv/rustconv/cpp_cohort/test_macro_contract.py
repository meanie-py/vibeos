from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

if __package__:
    from .macro_contract import audit, numeric_literal
else:
    from macro_contract import audit, numeric_literal


class MacroContractTest(unittest.TestCase):
    def test_literal_spellings(self) -> None:
        self.assertEqual(numeric_literal("12"), 12)
        self.assertEqual(numeric_literal("0x10UL"), 16)
        self.assertEqual(numeric_literal("077"), 63)
        self.assertEqual(numeric_literal("0b101"), 5)
        self.assertIsNone(numeric_literal("(A + 1)"))

    def test_compiler_active_and_unsupported(self) -> None:
        source = """#define COUNT 12
#define HEX 0x10U
#define EXPR (COUNT + 1)
#define FUNC(x) ((x) + 1)
#define MULTI 1 \\
  + 2
#if 0
#define OFF 9
#endif
int main() { return COUNT; }
"""
        rust = "const COUNT: usize = 12;\nconst HEX: u32 = 0x11;\n"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            cpp = root / "sample.cpp"
            candidate = root / "candidate.rs"
            cpp.write_text(source, encoding="utf-8")
            candidate.write_text(rust, encoding="utf-8")
            rows = {row["name"]: row for row in audit(cpp, candidate)["macros"]}
        self.assertEqual(rows["COUNT"]["candidate"], "same_value")
        self.assertEqual(rows["HEX"]["candidate"], "different_value")
        self.assertEqual(rows["EXPR"]["status"], "unsupported")
        self.assertEqual(rows["FUNC"]["status"], "unsupported")
        self.assertEqual(rows["MULTI"]["status"], "unsupported")
        self.assertEqual(rows["OFF"]["status"], "inactive")


if __name__ == "__main__":
    unittest.main()
