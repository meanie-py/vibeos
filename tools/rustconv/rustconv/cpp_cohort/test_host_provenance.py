"""Control for compiler-derived reference header provenance."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

if __package__:
    from .host_provenance import cpp_reference_provenance, digest
else:
    from host_provenance import cpp_reference_provenance, digest


class HostProvenanceTest(unittest.TestCase):
    def test_header_change_changes_preprocessed_reference(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            header = root / "local.h"
            source = root / "reference.cpp"
            source.write_text('#include "local.h"\nint value = VALUE;\n', encoding="utf-8")
            header.write_text("#define VALUE 17\n", encoding="utf-8")
            first = cpp_reference_provenance(source)
            self.assertIn(
                {"path": str(header), "sha256": digest(header.read_bytes())},
                first["headers"],
            )
            header.write_text("#define VALUE 19\n", encoding="utf-8")
            second = cpp_reference_provenance(source)
            self.assertNotEqual(first["preprocessed_sha256"], second["preprocessed_sha256"])
            self.assertEqual(first["source_sha256"], second["source_sha256"])


if __name__ == "__main__":
    unittest.main()
