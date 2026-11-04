"""QMP compatibility facade; framing and event retention live in Rust."""

from __future__ import annotations

import json
import threading

from tools.test_infra_compat import DedicatedSession, NativeError


class QmpError(Exception):
    def __init__(self, command, error):
        self.command = command
        self.error = error
        super().__init__(f"QMP {command}: {error.get('class', '?')}: {error.get('desc', '?')}")


class QmpClient:
    def __init__(self, *, _session=None, _vm_id=None):
        self._session = _session or DedicatedSession()
        self._owned = _session is None
        self._vm_id = _vm_id
        self._id = None if _vm_id is not None else self._session.call("qmp_create", {})["id"]
        self.lock = threading.RLock()
        self._closed = False

    def _call(self, suffix, **kwargs):
        op = f"qmp_{suffix}" if self._vm_id is None else f"vm_qmp_{suffix}"
        with self.lock:
            try:
                return self._session.call(
                    op, {"id": self._id if self._vm_id is None else self._vm_id, **kwargs}
                )
            except NativeError as exc:
                if "timed out" in str(exc).lower():
                    raise TimeoutError(str(exc)) from exc
                if exc.kind == "QmpError":
                    payload = json.loads(str(exc))
                    raise QmpError(payload["command"], payload["error"]) from exc
                raise QmpError(
                    kwargs.get("command", suffix), {"class": "NativeError", "desc": str(exc)}
                ) from exc

    def connect(self, path, timeout=10.0):
        return self._call("connect", path=str(path), timeout=timeout)

    def connect_tcp(self, host, port, timeout=10.0):
        return self._call("connect_tcp", host=host, port=port, timeout=timeout)

    def listen_tcp(self, host="127.0.0.1", port=0):
        return self._call("listen_tcp", host=host, port=port)

    def accept(self, timeout=30.0):
        return self._call("accept", timeout=timeout)

    def execute(self, command, **args):
        return self._call("execute", command=command, arguments=args)

    def hmp(self, command):
        return self._call("hmp", command=command)

    @property
    def events(self):
        return self._call("events")

    def saw_event(self, name):
        return self._call("saw_event", name=name)

    def wait_event(self, name, timeout):
        return self._call("wait_event", name=name, timeout=timeout)

    def close(self):
        if self._closed:
            return
        self._closed = True
        if self._owned:
            try:
                self._call("close")
            finally:
                self._session.close()
