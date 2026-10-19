#!/usr/bin/env python3
"""Compatibility entry point for native delay-import library generation."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from codegen_compat import main

if __name__ == "__main__":
    main("delaylib")
