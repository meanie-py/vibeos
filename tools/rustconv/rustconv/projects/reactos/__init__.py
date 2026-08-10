"""The ReactOS / vibeos project plugin."""

from .targets import (
    TARGETS,
    TargetResolveError,
    detect_target,
    hal_cmake_for_c,
    make_user_dll_target,
)

__all__ = [
    "TARGETS",
    "TargetResolveError",
    "detect_target",
    "hal_cmake_for_c",
    "make_user_dll_target",
]
