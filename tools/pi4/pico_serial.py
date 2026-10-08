"""Pi 4 serial console through the Pico 2 W's GPIO UART bridge.

Drop-in for Capture-RpiSerial.ps1 with the same bench contract, so netboot.sh and
serial-send.sh do not care which one runs:

  <bench>/serial-current.json   status LISTENING + the run directory
  <bench>/serial-<ts>/serial.bin every console byte, raw
  <bench>/serial-send.bin       bytes to send; deleted once written to the console

Runs in WSL under the picousb client venv (it imports pico_devbox). The token is
read from PICO_DEVICE_TOKEN_FILE by the client library and never printed.
Reconnects on its own: the Pico buffers the newest 128 KiB while no client is
attached and marks any gap in the stream as "[picousb: N console bytes lost]".

  ~/.venvs/picousb/bin/python tools/pi4/pico_serial.py <bench> [picousb-host]
"""

import asyncio
import json
import os
import pathlib
import signal
import sys
import time

from pico_devbox import DevBoxError
from pico_devbox.auth import read_token_file
from pico_devbox.serial import SerialConsole

SEND_POLL_S = 0.05
IDLE_RECYCLE_S = 120.0


class IdleConsole(Exception):
    """No byte for IDLE_RECYCLE_S: reconnect in case the connection is half-open."""


class Capture:
    def __init__(self, bench: pathlib.Path, host: str) -> None:
        self.bench, self.host = bench, host
        self.run_dir = bench / time.strftime("serial-%Y%m%d-%H%M%S")
        self.run_dir.mkdir()
        self.log = open(self.run_dir / "serial.bin", "xb")  # noqa: SIM115 - lives as long as the capture
        self.console: SerialConsole | None = None
        self.connected = False

    def state(self, status: str, **extra: object) -> None:
        record = {
            "status": status,
            "port": f"{self.host}:8082 (picousb)",
            "baud": 115200,
            "format": "8N1",
            "pid": os.getpid(),
            "started": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "directory": str(self.run_dir),
            "connected": self.connected,
            **extra,
        }
        tmp = self.bench / "serial-current.json.tmp"
        tmp.write_text(json.dumps(record, indent=2))
        tmp.replace(self.bench / "serial-current.json")

    async def pump_output(self, console: SerialConsole) -> None:
        # A reconnect can come up half-open: accepted, then silent for good. One
        # did for 11 hours while status still said connected, and a whole boot's
        # output was lost. So a quiet console is recycled; the Pico keeps the
        # newest 128 KiB while no client is attached, so nothing is lost.
        while True:
            try:
                data = await asyncio.wait_for(console.read(), IDLE_RECYCLE_S)
            except TimeoutError:
                raise IdleConsole from None
            self.log.write(data)
            self.log.flush()

    async def pump_input(self, console: SerialConsole) -> None:
        # serial-send.sh paces itself on this file disappearing, so consume it
        # only once its bytes are on their way to the target.
        path = self.bench / "serial-send.bin"
        while True:
            await asyncio.sleep(SEND_POLL_S)
            try:
                data = path.read_bytes()
            except FileNotFoundError:
                continue
            await console.write(data)
            path.unlink(missing_ok=True)

    async def run(self, token: str) -> None:
        delay = 1.0
        self.state("LISTENING")
        while True:
            try:
                async with SerialConsole(self.host, token) as console:
                    self.connected, delay = True, 1.0
                    self.state("LISTENING")
                    tasks = [
                        asyncio.create_task(self.pump_output(console)),
                        asyncio.create_task(self.pump_input(console)),
                    ]
                    try:
                        done, _ = await asyncio.wait(tasks, return_when=asyncio.FIRST_EXCEPTION)
                        for task in done:
                            task.result()
                    finally:
                        for task in tasks:
                            task.cancel()
            except IdleConsole:
                self.connected, delay = False, 1.0
                continue  # reconnect at once; a failure after this backs off from 1 s
            except DevBoxError:
                # A wrong token will not fix itself by retrying.
                self.connected = False
                self.state("FAILED", error="console authentication failed")
                raise
            except (OSError, TimeoutError) as error:
                if self.connected:
                    print(
                        f"pico-serial: console dropped ({type(error).__name__}); reconnecting",
                        file=sys.stderr,
                        flush=True,
                    )
                self.connected = False
                self.state("LISTENING", last_error=type(error).__name__)
            await asyncio.sleep(delay)
            delay = min(delay * 2, 8.0)


def main() -> None:
    if len(sys.argv) not in (2, 3):
        sys.exit(__doc__)
    bench = pathlib.Path(sys.argv[1])
    host = sys.argv[2] if len(sys.argv) == 3 else "picousb"
    token_file = os.environ.get(
        "PICO_DEVICE_TOKEN_FILE", str(pathlib.Path.home() / ".config/picousb/token")
    )
    token = read_token_file(token_file)
    capture = Capture(bench, host)
    print(f"LISTENING {host}:8082 -> {capture.run_dir}", flush=True)
    loop = asyncio.new_event_loop()
    task = loop.create_task(capture.run(token))
    for sig in (signal.SIGTERM, signal.SIGINT):
        loop.add_signal_handler(sig, task.cancel)
    try:
        loop.run_until_complete(task)
    except asyncio.CancelledError:
        capture.state("STOPPED")
    except Exception as error:
        capture.state("FAILED", error=type(error).__name__)
        raise
    finally:
        capture.log.close()


if __name__ == "__main__":
    main()
