"""Step-1 agent de-risk: did the livecd Run key launch notepad in-guest?

Boots i386, waits for desktop + extra time for explorer to process the
Run key (which cmd /c start notepad.exe), screendumps. A notepad window
appearing => CreateProcess/app-launch works in-guest, independent of the
broken host-input path => the snapshot-app vision is reachable via an
in-guest agent even before input is fixed.

    python -m tests.qemu.experiments.agent_check
"""

from __future__ import annotations

import time
from pathlib import Path

from tests.qemu.vm import PROJECT_DIR, QemuVM, get_arch_config


def main() -> None:
    cfg = get_arch_config("i386")
    cfg.desktop_settle_seconds = 25.0
    vm = QemuVM(cfg, Path("/tmp/agent_check_work"))
    print("=== boot i386 (livecd Run key -> start notepad.exe) ===")
    vm.start()
    vm.wait_for_boot()
    print("  desktop up; waiting 25s for explorer to process Run key + notepad to render...")
    time.sleep(25)

    ppm = vm.work_dir / "agent.ppm"
    vm.screendump(ppm)
    png = PROJECT_DIR / "qemu" / "i386_agent_notepad.png"
    try:
        from PIL import Image

        Image.open(ppm).save(png, "PNG")
        sz = png.stat().st_size
    except Exception as e:
        sz = -1
        print(f"  PIL save failed: {e}")
    print(f"  screenshot saved: {png} ({sz} bytes)")
    print("  >>> inspect the PNG: notepad window present == app-launch works")
    vm.stop()


if __name__ == "__main__":
    main()
