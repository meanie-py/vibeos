#!/usr/bin/env python3
"""Compatibility entry point for the native PE base-address generator."""

import os
import sys
from pathlib import Path

if __name__ == "__main__":
    tool = Path(__file__).resolve().parents[2] / "tools/codegen"
    args = sys.argv[1:]
    if any(arg.lower() in {"-help", "/help", "/h", "-h", "/?", "-?"} for arg in args):
        args = ["--help"]
    elif not args or args == ["-64"]:
        args.append(str(Path.cwd()))
    os.execv(str(tool), [str(tool), "baseaddress", *args])
