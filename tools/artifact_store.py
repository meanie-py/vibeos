"""Compatibility objects for native immutable-store locking and publication.

OS resources belong to dedicated native connections. Python callbacks keep their
original signatures; the native store owns validation, transactions and recovery.
"""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, NoReturn

from tools import build_infra_compat as bridge

SCHEMA = "vibeos-artifact-store-v1"


class StoreError(RuntimeError):
    pass


class CorruptStore(StoreError):
    pass


class ActionConflict(StoreError):
    pass


class BlobCollision(CorruptStore):
    pass


class SessionError(StoreError):
    pass


_ERRORS = {
    c.__name__: c for c in (StoreError, CorruptStore, ActionConflict, BlobCollision, SessionError)
}


def _raise(error) -> NoReturn:
    raise _ERRORS.get(error.kind, StoreError)(str(error)) from error


def _pure(op, **arguments) -> Any:
    try:
        return bridge.call("artifact_store_" + op, arguments)
    except bridge.NativeError as error:
        _raise(error)


def _strict(value):
    """Validate Python-to-JSON types before the native canonical encoder sees them."""
    if isinstance(value, dict):
        if any(not isinstance(key, str) for key in value):
            raise StoreError("JSON object keys must be strings")
        for item in value.values():
            _strict(item)
    elif isinstance(value, (list, tuple)):
        for item in value:
            _strict(item)
    elif value is not None and not isinstance(value, (str, bool, int, float)):
        raise StoreError(f"non-JSON descriptor value: {type(value).__name__}")


def canonical(value: Any) -> bytes:
    _strict(value)
    try:
        json.dumps(value, allow_nan=False)
        return _pure("canonical", value=value).encode("ascii")
    except (ValueError, TypeError) as error:
        raise StoreError(str(error)) from error


def action_key(descriptor: dict) -> str:
    if not isinstance(descriptor, dict):
        raise StoreError("action descriptor must be a JSON object")
    _strict(descriptor)
    try:
        json.dumps(descriptor, allow_nan=False)
    except (ValueError, TypeError) as error:
        raise StoreError(str(error)) from error
    return _pure("action_key", descriptor=descriptor)


def _key(value):
    return _pure("key", value=value)


def _name(value):
    return _pure("name", value=value)


def _owner():
    return _pure("owner", pid=os.getpid())


def _start_identity(pid):
    return _pure("start_identity", pid=pid)


def _owner_alive(owner):
    return _pure("owner_alive", owner=owner)


def _read(path):
    return bytes(_pure("read", path=path))


def _json(path):
    return _pure("read_json", path=path)


def _digest_file(path):
    return tuple(_pure("digest_file", path=path))


def _replace_json(path, value):
    return _pure("replace_json", path=path, value=value)


def _write_new(path, payload):
    return _pure("write_new", path=path, payload=payload)


def _rename_noreplace(source, destination):
    return _pure("rename_noreplace", source=source, destination=destination)


def _remove_private_tree(path):
    return _pure("remove_private_tree", path=path)


def _clone_or_copy(incoming, outgoing):
    result = subprocess.run(
        [
            str(bridge.REPO / "tools/build-infra"),
            "artifact-store-copy",
            str(incoming.fileno()),
            str(outgoing.fileno()),
        ],
        pass_fds=(incoming.fileno(), outgoing.fileno()),
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode:
        raise StoreError(result.stderr.strip())


class _Connection(bridge.DedicatedSession):
    def __init__(self):
        super().__init__("artifact-store-service")

    def store_call(self, op, arguments, callbacks=None) -> Any:
        try:
            return self.call(op, arguments, callbacks)
        except bridge.NativeError as error:
            _raise(error)


@dataclass(frozen=True)
class Output:
    path: Path
    role: str = "archive"


@dataclass
class BuildResult:
    outputs: dict[str, Output | Path]
    provenance: dict = field(default_factory=dict)
    dependency_actions: tuple[str, ...] = ()
    input_inventory: Any = None


@dataclass(frozen=True)
class Handle:
    action_key: str
    manifest: dict
    outputs: dict[str, Path]
    hit: bool
    verified_rebuild: bool = False
    session_verified: bool = False

    @property
    def blob_path(self):
        return Path(_pure("handle_blob_path", handle=self))


def _handle(value):
    return Handle(
        **{**value, "outputs": {key: Path(path) for key, path in value["outputs"].items()}}
    )


class Session:
    def __init__(self, store, *, token=None):
        self.store, self.owned = store, token is None
        self.owner = _owner()
        self.identifier = token.split(":", 1)[0] if token else ""
        self.token = token
        self.fd = None
        self.closed = False
        self._connection = None

    def __enter__(self):
        if self.closed:
            raise SessionError("session already closed")
        if self.fd is not None:
            raise SessionError("session already entered")
        if self.owned:
            self._connection = _Connection()
            try:
                data = self._connection.store_call(
                    "session_open", {"root": self.store.root, "owner": self.owner}
                )
            except BaseException:
                self._connection.close()
                raise
            self.token, self.identifier = data["token"], data["identifier"]
            self.fd = self._connection.fd
        else:
            self.store._call("attach", token=self.token)
        return self

    def __exit__(self, *_exception):
        if self.owned and self._connection is not None and self.fd is not None:
            if self.owner["pid"] != os.getpid():
                raise SessionError("fork child cannot close supervisor session")
            try:
                self._connection.store_call(
                    "session_close", {"root": self.store.root, "token": self.token}
                )
            finally:
                self._connection.close()
                self.fd = None
        self.closed = True

    def pin(self, key):
        return self.store.lookup(key, session=self)

    def bind(self, lane, consumer, key):
        return self.store.bind(lane, consumer, key, session=self)


class ConsumerGuard:
    def __init__(self, store, lane, consumer):
        self.store, self.lane, self.consumer = store, lane, consumer
        self.owner = os.getpid()
        self.fd = None
        self._connection = None
        self._identifier = None

    def __enter__(self):
        if self.fd is not None:
            raise StoreError("consumer guard already entered")
        self._connection = _Connection()
        try:
            self._identifier = self._connection.store_call(
                "guard_open",
                {"root": self.store.root, "lane": self.lane, "consumer": self.consumer},
            )
            self.fd = self._connection.fd
        except BaseException:
            self._connection.close()
            raise
        return self

    def __exit__(self, *_exception):
        if self.owner != os.getpid():
            raise StoreError("fork child cannot close consumer guard")
        if self._connection:
            try:
                self._connection.store_call(
                    "guard_close", {"root": self.store.root, "guard": self._identifier}
                )
            finally:
                self._connection.close()
                self.fd = None


class Store:
    def __init__(self, root):
        self.root = Path(root)
        self.base = self.root / "v1"
        self._call("init")

    def _call(self, op, **arguments) -> Any:
        with _Connection() as connection:
            return connection.store_call(op, {"root": self.root, **arguments})

    def _path(self, kind, key):
        return Path(_pure("path", root=self.root, kind=kind, key=key))

    def manifest_path(self, key):
        return self._action_path(key)

    def _action_path(self, key):
        return self._path("action", key)

    def _action_lock(self, key):
        return self._path("action_lock", key)

    def _blob(self, digest):
        return self._path("blob", digest)

    def _lease_path(self, identifier):
        return self._path("lease", identifier)

    def _lease_lock(self, identifier):
        return self._path("lease_lock", identifier)

    def _conflict_path(self, key):
        return self._path("conflict", key)

    def _ref_id(self, lane, consumer):
        return _pure("ref_id", root=self.root, lane=lane, consumer=consumer)

    def _ref_path(self, lane, consumer):
        return Path(_pure("ref_path", root=self.root, lane=lane, consumer=consumer))

    def session(self):
        return Session(self)

    def attach_session(self, token):
        self._call("attach", token=token)
        return Session(self, token=token)

    def lookup(self, key, *, session):
        self._validate_session(session)
        return _handle(self._call("lookup", key=key, token=session.token))

    def inspect_manifest(self, key, *, session=None):
        if session is not None:
            self._validate_session(session)
        return self._call("inspect", key=key, token=session.token if session else None)

    def _validate_session(self, session):
        if session.closed or session.store.root != self.root:
            raise SessionError("closed session or wrong store")

    def _build(self, descriptor, builder, session, *, repair_key=None, **options):
        self._validate_session(session)
        if descriptor is not None:
            canonical(
                descriptor
            )  # Validate transport types before the descriptor is frozen natively.
        with _Connection() as connection:
            request = {"root": self.root, "token": session.token, **options}
            started = connection.store_call(
                "begin_repair" if repair_key else "begin",
                {**request, **({"key": repair_key} if repair_key else {"descriptor": descriptor})},
            )
            if "handle" in started:
                return _handle(started["handle"])
            pending = True
            try:
                built = builder(Path(started["scratch"]))
                if not isinstance(built, BuildResult) or not built.outputs:
                    raise StoreError("builder must return nonempty BuildResult")
                outputs = {
                    name: value if isinstance(value, Output) else Output(Path(value))
                    for name, value in built.outputs.items()
                }
                for name, output in outputs.items():
                    if not isinstance(name, str) or not isinstance(output.role, str):
                        raise StoreError("output names and roles must be strings")
                _strict(built.provenance)
                _strict(built.input_inventory)
                json.dumps([built.provenance, built.input_inventory], allow_nan=False)
                payload = {
                    "outputs": outputs,
                    "provenance": built.provenance,
                    "dependency_actions": built.dependency_actions,
                    "input_inventory": built.input_inventory,
                }
                pending = False  # finish consumes its transaction even when publication fails.
                return _handle(
                    connection.store_call(
                        "finish",
                        {**request, "transaction": started["transaction"], "built": payload},
                    )
                )
            except BaseException as error:
                if pending:
                    connection.store_call(
                        "abort",
                        {
                            **request,
                            "transaction": started["transaction"],
                            "error_kind": type(error).__name__,
                            "message": str(error),
                        },
                    )
                raise

    def lookup_or_build(
        self,
        descriptor,
        builder,
        *,
        session,
        verify=False,
        verify_once_per_session=False,
        require_verified=False,
    ):
        return self._build(
            descriptor,
            builder,
            session,
            verify=verify,
            verify_once_per_session=verify_once_per_session,
            require_verified=require_verified,
        )

    def repair_missing(self, key, builder, *, session):
        return self._build(None, builder, session, repair_key=key)

    def bind(self, lane, consumer, key, *, session):
        self._validate_session(session)
        return Path(self._call("bind", lane=lane, consumer=consumer, key=key, token=session.token))

    def consumer_guard(self, lane, consumer):
        return ConsumerGuard(self, lane, consumer)

    def release_ref(self, lane, consumer, *, guard):
        if (
            guard.store.root != self.root
            or (guard.lane, guard.consumer) != (lane, consumer)
            or guard.fd is None
            or guard.owner != os.getpid()
            or guard._connection is None
        ):
            raise StoreError("release_ref requires the matching held consumer guard")
        guard._connection.store_call(
            "release_ref",
            {"root": self.root, "lane": lane, "consumer": consumer, "guard": guard._identifier},
        )

    def list_refs(self, lane):
        return self._call("list_refs", lane=lane)

    def restore_refs(self, lane, *, session):
        self._validate_session(session)
        return [
            _handle(value) for value in self._call("restore_refs", lane=lane, token=session.token)
        ]

    def status(self):
        return self._call("status")

    def gc(self, *, apply=False, grace_seconds=3600, ref_alive=None):
        callbacks = (
            {"ref_alive": lambda arguments: ref_alive(arguments["reference"])}
            if ref_alive
            else None
        )
        with _Connection() as connection:
            return connection.store_call(
                "gc",
                {
                    "root": self.root,
                    "apply": apply,
                    "grace_seconds": grace_seconds,
                    "ref_alive": ref_alive is not None,
                },
                callbacks=callbacks,
            )
