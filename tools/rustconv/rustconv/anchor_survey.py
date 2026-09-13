"""Free, reproducible c2rust capability sampling against this worktree's compile DBs.

Run with explicit source paths and --output report.json. The unpreprocessed source
inventory is reported separately: conditional definitions can explain missing names.
A successful compiler run is not a semantic-equivalence verdict.
"""

import argparse
import hashlib
import json
from pathlib import Path

from .c2rust_anchor import C2RustError, build_anchor_map
from .config import REACTOS_ROOT
from .scanner import scan_file


def survey(paths: list[Path], arches: list[str]) -> list[dict]:
    rows = []
    for source in paths:
        scan = scan_file(source)
        planned = {f.name for f in scan.functions}
        for arch in arches:
            row = {
                "source": str(source.relative_to(REACTOS_ROOT)),
                "arch": arch,
                "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                "raw_source_functions": sorted(planned),
                "unextracted": scan.unextracted,
            }
            try:
                anchors = build_anchor_map(source, arch=arch, strict=True, verbose=False)
                row.update(
                    status="partial" if planned - anchors.keys() else "anchored",
                    source_names_anchored=sorted(planned & anchors.keys()),
                    missing_raw_names=sorted(planned - anchors.keys()),
                )
            except C2RustError as error:
                row.update(status="refused", reason=str(error))
            rows.append(row)
            print(f"{arch} {row['source']}: {row['status']}", flush=True)
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sources", nargs="+")
    parser.add_argument(
        "--arches",
        nargs="+",
        choices=["i386", "amd64", "arm64"],
        default=["i386", "amd64", "arm64"],
    )
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    paths = [(REACTOS_ROOT / path).resolve() for path in args.sources]
    if any(not path.is_relative_to(REACTOS_ROOT.resolve()) for path in paths):
        parser.error("survey inputs must belong to this worktree")
    rows = survey(paths, args.arches)
    Path(args.output).write_text(json.dumps({"paid_calls": 0, "results": rows}, indent=2) + "\n")
    raise SystemExit(int(any(row["status"] != "anchored" for row in rows)))


if __name__ == "__main__":
    main()
