#!/usr/bin/env python3
"""Compatibility entry point; generation lives in tools/vibeos-codegen."""

from codegen_compat import main

if __name__ == "__main__":
    main("ioctl-consts")
