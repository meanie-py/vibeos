#!/usr/bin/env python3
"""Compatibility entry point; use `tools/codegen ntoskrnl-res`."""

from codegen_compat import main

if __name__ == "__main__":
    main("ntoskrnl-res")
