"""QMP event-retention controls over real protocol sockets."""

import json
import socket
import threading

from tests.qemu.qmp_client import QmpClient


def server(tmp_path, replies):
    path = tmp_path / "qmp.sock"
    listener = socket.socket(socket.AF_UNIX)
    listener.bind(str(path))
    listener.listen()
    failures = []

    def serve():
        try:
            conn, _ = listener.accept()
            with conn:
                stream = conn.makefile("rwb", buffering=0)
                stream.write(b'{"QMP":{}}\n')
                request = json.loads(stream.readline())
                assert request["execute"] == "qmp_capabilities"
                stream.write(b'{"return":{}}\n')
                for group in replies:
                    request = json.loads(stream.readline())
                    assert request["execute"].startswith("query-")
                    for value in group:
                        stream.write((json.dumps(value) + "\n").encode())
        except BaseException as exc:
            failures.append(exc)
        finally:
            listener.close()

    thread = threading.Thread(target=serve, daemon=True)
    thread.start()
    return path, thread, failures


def test_qmp_client_keeps_events_instead_of_discarding_them(tmp_path):
    path, thread, failures = server(
        tmp_path,
        [
            [{"event": "RESUME"}, {"return": {}}],
            [{"event": "STOP", "timestamp": {"seconds": 7}}, {"return": {"icount": 42}}],
        ],
    )
    client = QmpClient()
    try:
        client.connect(path)
        assert client.execute("query-first") == {"return": {}}
        assert client.execute("query-second") == {"return": {"icount": 42}}
        assert [event["event"] for event in client.events] == ["RESUME", "STOP"]
        assert client.saw_event("STOP") == {"event": "STOP", "timestamp": {"seconds": 7}}
        assert client.saw_event("SHUTDOWN") is None
    finally:
        client.close()
    thread.join(timeout=5)
    assert not thread.is_alive() and not failures


def test_qmp_event_retention_is_bounded(tmp_path):
    path, thread, failures = server(
        tmp_path, [[*({"event": f"E{i}"} for i in range(300)), {"return": {}}]]
    )
    client = QmpClient()
    try:
        client.connect(path)
        client.execute("query-tail")
        assert len(client.events) == 256
        assert client.saw_event("E299") is not None
        assert client.saw_event("E0") is None
    finally:
        client.close()
    thread.join(timeout=5)
    assert not thread.is_alive() and not failures
