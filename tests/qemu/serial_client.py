"""Compatibility objects for the native lifetime serial drainer."""

from __future__ import annotations

import os
import threading
import time

from tools.test_infra_compat import DedicatedSession, NativeError


class SerialClient:
    def __init__(self, *, _session=None, _vm_id=None):
        self._session = _session or DedicatedSession()
        self._owned = _session is None
        self._vm_id = _vm_id
        self._id = None if _vm_id is not None else self._session.call("serial_create", {})["id"]
        self._lock = threading.RLock()
        self._closed = False
        self.partial_output = ""
        self._alive = None

    def _call(self, suffix, **kwargs):
        if self._alive is not None:
            kwargs["alive"] = bool(self._alive())
        op = f"serial_{suffix}" if self._vm_id is None else f"vm_serial_{suffix}"
        with self._lock:
            try:
                return self._session.call(
                    op, {"id": self._id if self._vm_id is None else self._vm_id, **kwargs}
                )
            except NativeError as exc:
                if "timed out" in str(exc).lower() or "timeout" in str(exc).lower():
                    self.partial_output = self._state().get("partial_output", "")
                    raise TimeoutError(str(exc)) from exc
                if "closed" in str(exc).lower() or "exited" in str(exc).lower():
                    raise ConnectionError(str(exc)) from exc
                raise

    def listen(self, path):
        self._call("listen", path=str(path))

    def listen_tcp(self, host="127.0.0.1", port=0):
        return self._call("listen_tcp", host=host, port=port)

    def set_tee(self, path, header=""):
        self._call("tee", path=str(path), header=header)

    def follow_file(self, path, alive):
        self._alive = alive
        self._call("follow_file", path=str(path), owner_pid=os.getpid())

    def accept(self, timeout=30.0):
        self._call("accept", timeout=timeout)

    def read_until(self, marker, timeout=60.0):
        return self._call("read_until", marker=marker, timeout=timeout)

    def read_until_idle(self, idle_seconds=5.0, timeout=120.0, min_bytes=1024):
        return self._call(
            "read_until_idle", idle_seconds=idle_seconds, timeout=timeout, min_bytes=min_bytes
        )

    def read_available(self, timeout=0.1):
        return self._call("read_available", timeout=timeout)

    def send(self, data):
        self._call("send", data=list(data))

    def _state(self):
        return self._call("snapshot")

    def _snapshot(self):
        state = self._state()
        return state["text"], time.monotonic() - state["last_data_age"], state["dead"]

    @property
    def _buf(self):
        return self._state()["text"]

    @property
    def _pos(self):
        return self._state()["position"]

    @_pos.setter
    def _pos(self, value):
        self._call("set_position", position=value)

    @property
    def _last_data(self):
        return time.monotonic() - self._state()["last_data_age"]

    @property
    def _sock_dead(self):
        return self._state()["dead"]

    def close(self):
        if self._closed:
            return
        self._closed = True
        if self._owned:
            try:
                self._call("close")
            finally:
                self._session.close()
