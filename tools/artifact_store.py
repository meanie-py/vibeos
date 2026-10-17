"""Trusted local immutable-body store primitives; no compiler or hit activation.

Lock order: optional consumer guard -> action lock -> GC lock. GC NEVER waits
for an action lock. Compilation and output copying occur outside the GC lock.
Stable lock files are never deleted. Sessions own an OS lock through supervisor
exit; children attach with a capability and cannot close the supervisor lease.
Durable refs survive sessions and must be explicitly released under a consumer
guard plus the caller's build/maintenance interlock.
"""

from __future__ import annotations

import ctypes
import errno
import fcntl
import hashlib
import json
import math
import os
import secrets
import shutil
import stat
import time
import uuid
from collections.abc import Callable, Iterator
from contextlib import contextmanager
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath
from typing import Any

SCHEMA = "vibeos-artifact-store-v1"
AT_FDCWD = -100
RENAME_NOREPLACE = 1


class StoreError(RuntimeError):
    """Store operation failed; callers must not silently fall back."""


class CorruptStore(StoreError):
    """Indexed data is missing, malformed, or does not match its digest."""


class ActionConflict(StoreError):
    """Independent builds of one action produced different output bytes."""


class BlobCollision(CorruptStore):
    """Two different byte strings claimed the same blob identity."""


class SessionError(StoreError):
    """Supervisor lease/capability is absent, stale, or no longer protected."""


def canonical(value: Any) -> bytes:
    """Versioned manifests use strict JSON; absent and empty remain distinct."""

    def validate(item: Any) -> None:
        if isinstance(item, dict):
            if any(not isinstance(key, str) for key in item):
                raise StoreError("JSON object keys must be strings")
            for entry in item.values():
                validate(entry)
        elif isinstance(item, (list, tuple)):
            for entry in item:
                validate(entry)
        elif item is not None and not isinstance(item, (str, bool, int, float)):
            raise StoreError(f"non-JSON descriptor value: {type(item).__name__}")

    validate(value)
    try:
        return json.dumps(
            value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False
        ).encode()
    except (ValueError, TypeError) as error:
        raise StoreError(str(error)) from error


def action_key(descriptor: dict) -> str:
    if not isinstance(descriptor, dict):
        raise StoreError("action descriptor must be a JSON object")
    return hashlib.sha256(canonical(descriptor)).hexdigest()


def _key(value: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(c not in "0123456789abcdef" for c in value)
    ):
        raise StoreError(f"invalid SHA-256 identity: {value!r}")
    return value


def _name(value: str) -> str:
    path = PurePosixPath(value)
    if not value or path.is_absolute() or ".." in path.parts or str(path) != value or value == ".":
        raise StoreError(f"invalid output name: {value!r}")
    return value


def _start_identity(pid: int) -> dict | None:
    try:
        # /proc comm may contain ')' and spaces; the final ')' ends field 2.
        fields = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
        return {
            "boot_id": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
            "start_ticks": fields[19],
        }
    except (OSError, IndexError):
        return None


def _owner() -> dict:
    identity = _start_identity(os.getpid())
    if identity is None:
        raise StoreError("Linux process start identity unavailable")
    return {"pid": os.getpid(), "start_identity": identity}


def _owner_alive(owner: dict) -> bool:
    if not isinstance(owner, dict) or set(owner) != {"pid", "start_identity"}:
        raise CorruptStore("invalid process owner identity")
    identity = owner["start_identity"]
    if (
        type(owner["pid"]) is not int
        or owner["pid"] <= 0
        or not isinstance(identity, dict)
        or set(identity) != {"boot_id", "start_ticks"}
        or any(not isinstance(value, str) or not value for value in identity.values())
    ):
        raise CorruptStore("invalid process start identity")
    return _start_identity(owner["pid"]) == identity


def _mkdir(path: Path) -> None:
    missing = []
    current = path
    while not _exists(current):
        missing.append(current)
        current = current.parent
    if not stat.S_ISDIR(current.lstat().st_mode):
        raise CorruptStore(f"store directory ancestor is not a real directory: {current}")
    for directory in reversed(missing):
        directory.mkdir(exist_ok=True)
        if not stat.S_ISDIR(directory.lstat().st_mode):
            raise CorruptStore(f"store directory is not a real directory: {directory}")
        _fsync_dir(directory)
        _fsync_dir(directory.parent)
    if not stat.S_ISDIR(path.lstat().st_mode):
        raise CorruptStore(f"store directory is not a real directory: {path}")


def _fsync_dir(path: Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _exists(path: Path) -> bool:
    return os.path.lexists(path)


def _open_regular(path: Path, flags: int = os.O_RDONLY) -> int:
    try:
        descriptor = os.open(path, flags | os.O_CLOEXEC | os.O_NOFOLLOW)
    except OSError as error:
        raise CorruptStore(f"cannot open regular store file {path}: {error}") from error
    if not stat.S_ISREG(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise CorruptStore(f"not a regular file: {path}")
    return descriptor


def _stamp(info: os.stat_result) -> tuple:
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def _read(path: Path) -> bytes:
    descriptor = _open_regular(path)
    with os.fdopen(descriptor, "rb") as source:
        before = os.fstat(source.fileno())
        data = source.read()
        if _stamp(before) != _stamp(os.fstat(source.fileno())):
            raise CorruptStore(f"file changed during read: {path}")
        return data


def _json(path: Path) -> dict:
    raw = _read(path)
    try:
        value = json.loads(raw)
        if not isinstance(value, dict) or canonical(value) != raw:
            raise ValueError("noncanonical JSON object")
        return value
    except (ValueError, UnicodeDecodeError) as error:
        raise CorruptStore(f"invalid JSON at {path}: {error}") from error


def _digest_file(path: Path) -> tuple[str, int]:
    descriptor = _open_regular(path)
    with os.fdopen(descriptor, "rb") as source:
        before = os.fstat(source.fileno())
        digest = hashlib.sha256()
        size = 0
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
            size += len(block)
        if _stamp(before) != _stamp(os.fstat(source.fileno())):
            raise CorruptStore(f"file changed during hashing: {path}")
        return digest.hexdigest(), size


def _same_bytes(first: Path, second: Path) -> bool:
    with (
        os.fdopen(_open_regular(first), "rb") as left,
        os.fdopen(_open_regular(second), "rb") as right,
    ):
        while True:
            a, b = left.read(1024 * 1024), right.read(1024 * 1024)
            if a != b:
                return False
            if not a:
                return True


def _rename_noreplace(source: Path, destination: Path) -> bool:
    """Atomic Linux no-replace rename, including directories; never hardlinks."""
    library = ctypes.CDLL(None, use_errno=True)
    try:
        rename = library.renameat2
    except AttributeError as error:
        raise StoreError("local store requires Linux renameat2(RENAME_NOREPLACE)") from error
    rename.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
    rename.restype = ctypes.c_int
    result = rename(
        AT_FDCWD, os.fsencode(source), AT_FDCWD, os.fsencode(destination), RENAME_NOREPLACE
    )
    if result == 0:
        _fsync_dir(destination.parent)
        if source.parent != destination.parent:
            _fsync_dir(source.parent)
        return True
    error = ctypes.get_errno()
    if error == errno.EEXIST:
        return False
    raise StoreError(f"atomic no-replace publication failed: {os.strerror(error)}")


def _write_new(path: Path, payload: bytes) -> None:
    descriptor = os.open(
        path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600
    )
    with os.fdopen(descriptor, "wb") as output:
        output.write(payload)
        output.flush()
        os.fsync(output.fileno())
    os.chmod(path, 0o444)


def _remove_private_tree(path: Path) -> None:
    """Delete owned scratch after workers exit, including read-only captures.

    chmod only real directories inside this private tree; symlinks are unlinked
    by rmtree, never followed. Immutable store blobs are outside these trees.
    """
    if not stat.S_ISDIR(path.lstat().st_mode):
        raise CorruptStore(f"scratch root is not a real directory: {path}")
    os.chmod(path, stat.S_IMODE(path.stat().st_mode) | 0o700)
    for current, directories, _files in os.walk(path, followlinks=False):
        for name in directories:
            child = Path(current) / name
            info = child.lstat()
            if stat.S_ISDIR(info.st_mode):
                os.chmod(child, stat.S_IMODE(info.st_mode) | 0o700)
    shutil.rmtree(path)


def _replace_json(path: Path, value: dict) -> None:
    temporary = path.with_name("." + path.name + "." + uuid.uuid4().hex)
    _write_new(temporary, canonical(value))
    os.replace(temporary, path)  # only mutable lease/ref records, under GC lock
    _fsync_dir(path.parent)


@contextmanager
def _lock(path: Path, *, nonblocking: bool = False) -> Iterator[int]:
    descriptor = os.open(path, os.O_CREAT | os.O_RDWR | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600)
    try:
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            raise CorruptStore(f"lock is not regular: {path}")
        fcntl.flock(descriptor, fcntl.LOCK_EX | (fcntl.LOCK_NB if nonblocking else 0))
        yield descriptor
    finally:
        os.close(descriptor)


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
    def blob_path(self) -> Path:
        archives = [
            self.outputs[item["name"]]
            for item in self.manifest["outputs"]
            if item["role"] == "archive"
        ]
        if len(archives) != 1:
            raise StoreError("handle does not have exactly one archive output")
        return archives[0]


class Session:
    def __init__(self, store: Store, *, token: str | None = None):
        self.store = store
        self.owner = _owner()
        self.owned = token is None
        self.identifier = uuid.uuid4().hex if token is None else token.split(":", 1)[0]
        self.token = self.identifier + ":" + secrets.token_hex(32) if token is None else token
        self.fd: int | None = None
        self.closed = False

    def __enter__(self) -> Session:
        if self.closed:
            raise SessionError("session already closed")
        if self.owned:
            if self.fd is not None:
                raise SessionError("session already entered")
            lock_path = self.store._lease_lock(self.identifier)
            self.fd = os.open(
                lock_path, os.O_CREAT | os.O_RDWR | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600
            )
            fcntl.flock(self.fd, fcntl.LOCK_EX)
            with self.store._gc_lock():
                record = {
                    "schema": SCHEMA,
                    "id": self.identifier,
                    "owner": self.owner,
                    "capability_sha256": hashlib.sha256(self.token.encode()).hexdigest(),
                    "actions": [],
                    "forced_actions": {},
                }
                _replace_json(self.store._lease_path(self.identifier), record)
        else:
            with self.store._gc_lock():
                self.store._session_record(self)
        return self

    def __exit__(self, *_exception: Any) -> None:
        if self.owned and self.fd is not None:
            if os.getpid() != self.owner["pid"]:
                raise SessionError("fork child cannot close supervisor session")
            with self.store._gc_lock():
                self.store._lease_path(self.identifier).unlink(missing_ok=True)
                _fsync_dir(self.store.base / "leases")
            os.close(self.fd)
            self.fd = None
        self.closed = True

    def pin(self, key: str) -> Handle:
        return self.store.lookup(key, session=self)

    def bind(self, lane: str, consumer: str, key: str) -> Path:
        return self.store.bind(lane, consumer, key, session=self)


class ConsumerGuard:
    def __init__(self, store: Store, lane: str, consumer: str):
        self.store, self.lane, self.consumer = store, lane, consumer
        self.owner = os.getpid()
        self.fd: int | None = None
        self._context: Any = None

    def __enter__(self) -> ConsumerGuard:
        if self.fd is not None:
            raise StoreError("consumer guard already entered")
        self._context = _lock(
            self.store.base
            / "locks"
            / ("consumer-" + self.store._ref_id(self.lane, self.consumer) + ".lock")
        )
        self.fd = self._context.__enter__()
        return self

    def __exit__(self, *exception: Any) -> None:
        self._context.__exit__(*exception)
        self.fd = None


class Store:
    def __init__(self, root: Path):
        root = Path(root)
        if not root.is_absolute():
            raise StoreError("store root must be explicitly absolute")
        self.root = root
        self.base = root / "v1"
        for path in (
            root,
            self.base,
            *(
                self.base / name
                for name in (
                    "blobs",
                    "blobs/sha256",
                    "actions",
                    "locks",
                    "staging",
                    "leases",
                    "refs",
                    "quarantine",
                )
            ),
        ):
            _mkdir(path)

    def manifest_path(self, key: str) -> Path:
        """Return a receipt location; use session-bound lookup to verify or read outputs."""
        return self._action_path(key)

    def _gc_lock(self):
        return _lock(self.base / "locks/gc.lock")

    def _action_path(self, key: str) -> Path:
        return self.base / "actions" / (_key(key) + ".json")

    def _action_lock(self, key: str) -> Path:
        return self.base / "locks" / (_key(key) + ".lock")

    def _blob(self, digest: str) -> Path:
        return self.base / "blobs/sha256" / _key(digest)[:2] / digest

    def _lease_path(self, identifier: str) -> Path:
        if len(identifier) != 32 or any(c not in "0123456789abcdef" for c in identifier):
            raise SessionError("invalid supervisor session id")
        return self.base / "leases" / (identifier + ".json")

    def _lease_lock(self, identifier: str) -> Path:
        self._lease_path(identifier)
        return self.base / "locks" / ("lease-" + identifier + ".lock")

    def _conflict_path(self, key: str) -> Path:
        return self.base / "quarantine" / _key(key) / "conflict.json"

    def session(self) -> Session:
        return Session(self)

    def attach_session(self, token: str) -> Session:
        if not isinstance(token, str) or ":" not in token:
            raise SessionError("invalid session capability")
        session = Session(self, token=token)
        with self._gc_lock():
            self._session_record(session)
        return session

    def _lock_held(self, path: Path) -> bool:
        if not _exists(path):
            return False
        try:
            with _lock(path, nonblocking=True):
                return False
        except BlockingIOError:
            return True

    def _validate_lease(self, record: dict) -> None:
        expected = {"schema", "id", "owner", "capability_sha256", "actions", "forced_actions"}
        if set(record) != expected or record.get("schema") != SCHEMA:
            raise CorruptStore("invalid supervisor lease fields/schema")
        self._lease_path(record["id"])
        _owner_alive(record["owner"])
        _key(record["capability_sha256"])
        actions, forced = record["actions"], record["forced_actions"]
        if not isinstance(actions, list) or any(not isinstance(key, str) for key in actions):
            raise CorruptStore("invalid supervisor pin inventory")
        if actions != sorted(set(actions)):
            raise CorruptStore("noncanonical supervisor pin inventory")
        for key in actions:
            _key(key)
        if not isinstance(forced, dict) or any(
            key not in actions or status not in {"built", "verified"}
            for key, status in forced.items()
        ):
            raise CorruptStore("invalid supervisor forced-build inventory")

    def _session_record(self, session: Session) -> dict:
        if session.store.root != self.root or session.closed:
            raise SessionError("closed session or wrong store")
        try:
            record = _json(self._lease_path(session.identifier))
        except CorruptStore as error:
            raise SessionError("supervisor session no longer exists") from error
        self._validate_lease(record)
        if record["id"] != session.identifier:
            raise SessionError("invalid supervisor lease identity")
        if record.get("capability_sha256") != hashlib.sha256(session.token.encode()).hexdigest():
            raise SessionError("supervisor capability mismatch")
        if not _owner_alive(record["owner"]) or not self._lock_held(
            self._lease_lock(session.identifier)
        ):
            raise SessionError("supervisor owner is dead/reused or its OS lease lock is not held")
        if session.owned and os.getpid() != session.owner["pid"]:
            raise SessionError("fork child must attach to supervisor capability")
        return record

    def _pin_locked(self, session: Session, key: str) -> None:
        record = self._session_record(session)
        record["actions"] = sorted(set(record["actions"]) | {_key(key)})
        _replace_json(self._lease_path(session.identifier), record)

    def _mark_forced_locked(self, session: Session, key: str, status: str) -> None:
        record = self._session_record(session)
        record.setdefault("forced_actions", {})[key] = status
        _replace_json(self._lease_path(session.identifier), record)

    def _quarantine_corruption_locked(self, key: str, error: CorruptStore) -> None:
        """Copy evidence without moving/deleting an object a linker may be reading."""
        folder = self.base / "quarantine" / key / ("corruption-" + uuid.uuid4().hex)
        _mkdir(folder)
        evidence = {
            "schema": SCHEMA,
            "action_key": key,
            "error": str(error),
            "observer": _owner(),
            "created_ns": time.time_ns(),
            "outputs": [],
        }
        manifest_path = self._action_path(key)
        try:
            raw = _read(manifest_path)
            _write_new(folder / "manifest.raw", raw)
            parsed = json.loads(raw)
        except (StoreError, OSError, ValueError, UnicodeDecodeError) as manifest_error:
            evidence["manifest_read_error"] = str(manifest_error)
            parsed = {}
        outputs = parsed.get("outputs") if isinstance(parsed, dict) else None
        for output in outputs if isinstance(outputs, list) else []:
            if not isinstance(output, dict):
                continue
            identity = output.get("sha256")
            if not isinstance(identity, str):
                continue
            try:
                digest = _key(identity)
            except StoreError:
                continue
            blob = self._blob(digest)
            observed = {"expected_sha256": digest, "path": str(blob), "exists": _exists(blob)}
            if _exists(blob):
                try:
                    identity = _digest_file(blob)
                    observed.update({"observed_sha256": identity[0], "observed_bytes": identity[1]})
                    if identity != (digest, output.get("bytes")):
                        # Preserve the offending bytes independently; original stays put.
                        retained = folder / (digest + ".corrupt")
                        if not _exists(retained):
                            _write_new(retained, _read(blob))
                        observed["retained_file"] = retained.name
                except (StoreError, OSError) as blob_error:
                    observed["read_error"] = str(blob_error)
            evidence["outputs"].append(observed)
        _write_new(folder / "diagnostic.json", canonical(evidence))
        _fsync_dir(folder)

    def _manifest_locked(
        self,
        key: str,
        *,
        missing_ok: bool = False,
        allow_missing_blobs: bool = False,
        verify_blobs: bool = True,
    ) -> dict | None:
        key = _key(key)
        try:
            return self._read_manifest_locked(
                key,
                missing_ok=missing_ok,
                allow_missing_blobs=allow_missing_blobs,
                verify_blobs=verify_blobs,
            )
        except CorruptStore as error:
            try:
                self._quarantine_corruption_locked(key, error)
            except (StoreError, OSError) as evidence_error:
                raise CorruptStore(
                    f"{error}; corruption evidence failed: {evidence_error}"
                ) from error
            raise

    def _read_manifest_locked(
        self,
        key: str,
        *,
        missing_ok: bool = False,
        allow_missing_blobs: bool = False,
        verify_blobs: bool = True,
    ) -> dict | None:
        key = _key(key)
        if _exists(self._conflict_path(key)):
            raise ActionConflict(f"action {key} is marked conflicting: {self._conflict_path(key)}")
        path = self._action_path(key)
        if not _exists(path):
            if missing_ok:
                return None
            raise CorruptStore(f"indexed/referenced action is missing: {key}")
        manifest = _json(path)
        expected_fields = {
            "schema",
            "action_key",
            "descriptor",
            "input_inventory",
            "configuration",
            "outputs",
            "dependency_actions",
            "publication",
        }
        if set(manifest) != expected_fields or not isinstance(manifest.get("descriptor"), dict):
            raise CorruptStore(f"invalid manifest fields: {path}")
        if (
            manifest.get("schema") != SCHEMA
            or manifest.get("action_key") != key
            or action_key(manifest["descriptor"]) != key
        ):
            raise CorruptStore(f"action identity/schema mismatch: {path}")
        outputs = manifest.get("outputs")
        if not isinstance(outputs, list) or not outputs:
            raise CorruptStore(f"missing output inventory: {path}")
        names = []
        for output in outputs:
            try:
                names.append(_name(output["name"]))
                _key(output["sha256"])
                if not isinstance(output["role"], str) or not output["role"]:
                    raise StoreError("invalid role")
                if type(output["bytes"]) is not int or output["bytes"] < 0:
                    raise StoreError("invalid size")
            except (KeyError, TypeError, StoreError) as error:
                raise CorruptStore(f"invalid output inventory: {path}") from error
            blob = self._blob(output["sha256"])
            if not verify_blobs:
                continue
            if allow_missing_blobs and not _exists(blob):
                continue
            if _digest_file(blob) != (output["sha256"], output["bytes"]):
                raise CorruptStore(f"blob digest/size mismatch: {self._blob(output['sha256'])}")
        if names != sorted(set(names)):
            raise CorruptStore(f"duplicate/unsorted output names: {path}")
        dependencies = manifest.get("dependency_actions")
        if not isinstance(dependencies, list) or dependencies != sorted(set(dependencies)):
            raise CorruptStore(f"invalid dependency action inventory: {path}")
        for dependency in dependencies:
            _key(dependency)
        return manifest

    def inspect_manifest(self, key: str, *, session: Session | None = None) -> dict:
        """Validate metadata identity/schema only; this NEVER certifies blob bytes.

        Intended for explicit repair planning. No archive handle is returned.
        A session optionally pins the action while its missing outputs are rebuilt.
        """
        with self._gc_lock():
            manifest = self._manifest_locked(key, verify_blobs=False)
            assert manifest is not None
            if session is not None:
                self._pin_locked(session, key)
            return manifest

    def _verify_tree_locked(
        self, key: str, cache: dict | None = None, visiting: set | None = None
    ) -> dict:
        cache = {} if cache is None else cache
        visiting = set() if visiting is None else visiting
        if key in visiting:
            raise CorruptStore(f"cyclic dependency action references: {key}")
        if key in cache:
            return cache[key]
        visiting.add(key)
        manifest = self._manifest_locked(key)
        assert manifest is not None
        for dependency in manifest["dependency_actions"]:
            self._verify_tree_locked(dependency, cache, visiting)
        visiting.remove(key)
        cache[key] = manifest
        return manifest

    def _handle(
        self, manifest: dict, hit: bool, verified: bool = False, session_verified: bool = False
    ) -> Handle:
        return Handle(
            manifest["action_key"],
            manifest,
            {output["name"]: self._blob(output["sha256"]) for output in manifest["outputs"]},
            hit,
            verified,
            session_verified,
        )

    def lookup(self, key: str, *, session: Session) -> Handle:
        with self._gc_lock():
            manifest = self._verify_tree_locked(key)
            self._pin_locked(session, key)
            return self._handle(manifest, True)

    def lookup_or_build(
        self,
        descriptor: dict,
        builder: Callable[[Path], BuildResult],
        *,
        session: Session,
        verify: bool = False,
        verify_once_per_session: bool = False,
        require_verified: bool = False,
    ) -> Handle:
        if verify_once_per_session and not verify:
            raise StoreError("verify_once_per_session requires verify=True")
        if require_verified and not verify:
            raise StoreError("require_verified requires verify=True")
        # Freeze caller-owned mutable descriptor before invoking a callback.
        descriptor = json.loads(canonical(descriptor))
        key = action_key(descriptor)
        with _lock(self._action_lock(key)):
            with self._gc_lock():
                self._pin_locked(session, key)  # inflight producer root, even before index exists
                previous = self._manifest_locked(key, missing_ok=True)
                if previous is not None:
                    previous = self._verify_tree_locked(key)
                    forced = self._session_record(session).get("forced_actions", {}).get(key)
                    satisfied = forced == "verified" or (forced == "built" and not require_verified)
                    if not verify or (verify_once_per_session and satisfied):
                        return self._handle(previous, True, session_verified=forced == "verified")
            stage = self.base / "staging" / (key + "-" + uuid.uuid4().hex)
            stage.mkdir()
            _write_new(
                stage / "producer.json",
                canonical({"schema": SCHEMA, "action_key": key, "owner": _owner()}),
            )
            scratch = stage / "worker"
            scratch.mkdir()
            try:
                built = builder(scratch)
                if not isinstance(built, BuildResult) or not built.outputs:
                    raise StoreError("builder must return nonempty BuildResult")
                candidate = self._prepare(stage, scratch, key, descriptor, built)
                with self._gc_lock():
                    self._session_record(session)  # do not publish for a dead supervisor
                    previous = self._manifest_locked(key, missing_ok=True)
                    if previous is not None:
                        same_identity = (previous["outputs"], previous["dependency_actions"]) == (
                            candidate["outputs"],
                            candidate["dependency_actions"],
                        )
                        same_bytes = same_identity and all(
                            _same_bytes(
                                stage / ("output-" + str(index)), self._blob(output["sha256"])
                            )
                            for index, output in enumerate(candidate["outputs"])
                        )
                        if not same_bytes:
                            self._quarantine_conflict(stage, key, previous, candidate)
                            raise ActionConflict(
                                f"independent output conflict for {key}; evidence retained"
                            )
                        self._pin_locked(session, key)
                        if verify:
                            self._mark_forced_locked(session, key, "verified")
                        handle = self._handle(previous, False, True, True)
                    else:
                        for dependency in candidate["dependency_actions"]:
                            self._verify_tree_locked(dependency)
                        self._publish_outputs(stage, candidate)
                        temporary = stage / "action.json"
                        _write_new(temporary, canonical(candidate))
                        if not _rename_noreplace(temporary, self._action_path(key)):
                            raise CorruptStore(
                                "action index appeared despite exclusive producer lock"
                            )
                        self._pin_locked(session, key)
                        if verify:
                            self._mark_forced_locked(session, key, "built")
                        handle = self._handle(candidate, False)
                _remove_private_tree(stage)
                _fsync_dir(self.base / "staging")
                return handle
            except BaseException as error:
                if stage.exists():
                    _replace_json(
                        stage / "failure.json",
                        {"schema": SCHEMA, "error": type(error).__name__, "message": str(error)},
                    )
                raise

    def repair_missing(
        self, key: str, builder: Callable[[Path], BuildResult], *, session: Session
    ) -> Handle:
        """Explicitly reproduce absent blobs, preserving the indexed manifest.

        Existing wrong-hash bytes/schema are corruption, never a repair request.
        The caller must prove this builder uses the indexed source action (or a
        retained historical snapshot). Outputs must reproduce every indexed hash.
        """
        key = _key(key)
        with _lock(self._action_lock(key)):
            with self._gc_lock():
                original = self._manifest_locked(key, allow_missing_blobs=True)
                assert original is not None
                if all(_exists(self._blob(output["sha256"])) for output in original["outputs"]):
                    raise StoreError("repair_missing requires at least one absent indexed blob")
                self._pin_locked(session, key)
            stage = self.base / "staging" / (key + "-" + uuid.uuid4().hex)
            stage.mkdir()
            _write_new(
                stage / "producer.json",
                canonical({"schema": SCHEMA, "action_key": key, "owner": _owner()}),
            )
            scratch = stage / "worker"
            scratch.mkdir()
            try:
                built = builder(scratch)
                if not isinstance(built, BuildResult) or not built.outputs:
                    raise StoreError("repair builder must return nonempty BuildResult")
                candidate = self._prepare(stage, scratch, key, original["descriptor"], built)
                with self._gc_lock():
                    self._session_record(session)
                    current = self._manifest_locked(key, allow_missing_blobs=True)
                    if current != original:
                        raise CorruptStore("indexed manifest changed during explicit repair")
                    identical = (original["outputs"], original["dependency_actions"]) == (
                        candidate["outputs"],
                        candidate["dependency_actions"],
                    )
                    if identical:
                        identical = all(
                            not _exists(self._blob(output["sha256"]))
                            or _same_bytes(
                                stage / ("output-" + str(index)), self._blob(output["sha256"])
                            )
                            for index, output in enumerate(candidate["outputs"])
                        )
                    if not identical:
                        self._quarantine_conflict(stage, key, original, candidate)
                        raise ActionConflict(
                            f"repair outputs disagree with indexed identity for {key}"
                        )
                    for dependency in original["dependency_actions"]:
                        self._verify_tree_locked(dependency)
                    self._publish_outputs(stage, candidate)
                    self._pin_locked(session, key)
                    self._mark_forced_locked(session, key, "verified")
                    handle = self._handle(original, False, True, True)
                _remove_private_tree(stage)
                _fsync_dir(self.base / "staging")
                return handle
            except BaseException as error:
                if stage.exists():
                    _replace_json(
                        stage / "failure.json",
                        {"schema": SCHEMA, "error": type(error).__name__, "message": str(error)},
                    )
                raise

    def _prepare(
        self, stage: Path, scratch: Path, key: str, descriptor: dict, built: BuildResult
    ) -> dict:
        outputs = []
        for name, specification in sorted(built.outputs.items()):
            _name(name)
            output = (
                specification if isinstance(specification, Output) else Output(Path(specification))
            )
            source = Path(output.path)
            if not source.is_absolute():
                source = scratch / source
            resolved = source.resolve()
            if scratch != resolved and scratch not in resolved.parents:
                raise StoreError(f"builder output escapes private scratch: {source}")
            if not isinstance(output.role, str) or not output.role:
                raise StoreError("output role must be nonempty")
            target = stage / ("output-" + str(len(outputs)))
            descriptor_fd = _open_regular(source)
            with os.fdopen(descriptor_fd, "rb") as incoming, target.open("xb") as outgoing:
                before = os.fstat(incoming.fileno())
                shutil.copyfileobj(incoming, outgoing, length=1024 * 1024)
                if _stamp(before) != _stamp(os.fstat(incoming.fileno())):
                    raise StoreError(f"builder output changed during copy: {source}")
                outgoing.flush()
                os.fsync(outgoing.fileno())
            os.chmod(target, 0o444)
            digest, size = _digest_file(target)
            outputs.append({"name": name, "role": output.role, "sha256": digest, "bytes": size})
        dependencies = sorted({_key(value) for value in built.dependency_actions})
        if key in dependencies:
            raise StoreError("action cannot depend on itself")
        manifest = {
            "schema": SCHEMA,
            "action_key": key,
            "descriptor": descriptor,
            "input_inventory": built.input_inventory
            if built.input_inventory is not None
            else descriptor.get("packages", {}),
            "configuration": descriptor.get("context", {}),
            "outputs": outputs,
            "dependency_actions": dependencies,
            "publication": {
                "producer": _owner(),
                "created_ns": time.time_ns(),
                "provenance": built.provenance,
            },
        }
        canonical(manifest)
        _fsync_dir(stage)
        return manifest

    def _publish_outputs(self, stage: Path, manifest: dict) -> None:
        for index, output in enumerate(manifest["outputs"]):
            candidate = stage / ("output-" + str(index))
            destination = self._blob(output["sha256"])
            _mkdir(destination.parent)
            if not _rename_noreplace(candidate, destination):
                if _digest_file(destination) != (output["sha256"], output["bytes"]):
                    raise CorruptStore(f"existing blob has wrong identity: {destination}")
                if not _same_bytes(candidate, destination):
                    raise BlobCollision(f"different bytes claim digest {output['sha256']}")
                candidate.unlink()
                _fsync_dir(stage)

    def _quarantine_conflict(self, stage: Path, key: str, previous: dict, candidate: dict) -> None:
        folder = self.base / "quarantine" / key
        _mkdir(folder)
        retained = folder / stage.name
        if not _rename_noreplace(stage, retained):
            raise CorruptStore("quarantine destination already exists")
        temporary = folder / (".conflict-" + uuid.uuid4().hex)
        _write_new(
            temporary,
            canonical(
                {
                    "schema": SCHEMA,
                    "action_key": key,
                    "previous": previous,
                    "candidate": candidate,
                    "candidate_directory": retained.name,
                }
            ),
        )
        if not _rename_noreplace(temporary, folder / "conflict.json"):
            temporary.unlink()

    def _ref_id(self, lane: str, consumer: str) -> str:
        if not isinstance(lane, str) or not lane or not isinstance(consumer, str) or not consumer:
            raise StoreError("lane and consumer must be nonempty logical identities")
        return hashlib.sha256(canonical([lane, consumer])).hexdigest()

    def _ref_path(self, lane: str, consumer: str) -> Path:
        return self.base / "refs" / (self._ref_id(lane, consumer) + ".json")

    def consumer_guard(self, lane: str, consumer: str) -> ConsumerGuard:
        return ConsumerGuard(self, lane, consumer)

    def bind(self, lane: str, consumer: str, key: str, *, session: Session) -> Path:
        with self.consumer_guard(lane, consumer), self._gc_lock():
            self._verify_tree_locked(key)
            self._pin_locked(session, key)
            path = self._ref_path(lane, consumer)
            _replace_json(
                path,
                {"schema": SCHEMA, "lane": lane, "consumer": consumer, "action_key": _key(key)},
            )
            return path

    def release_ref(self, lane: str, consumer: str, *, guard: ConsumerGuard) -> None:
        # This OS guard serializes bind/release. The CLI must additionally hold
        # its real build/maintenance interlock; this primitive cannot know Cargo.
        if (
            guard.store.root != self.root
            or (guard.lane, guard.consumer) != (lane, consumer)
            or guard.fd is None
            or guard.owner != os.getpid()
        ):
            raise StoreError("release_ref requires the matching held consumer guard")
        os.fstat(guard.fd)
        with self._gc_lock():
            self._ref_path(lane, consumer).unlink(missing_ok=True)
            _fsync_dir(self.base / "refs")

    def list_refs(self, lane: str) -> list[dict]:
        """Return validated receipt records; referenced blob integrity is separate."""
        if not isinstance(lane, str) or not lane:
            raise StoreError("lane must be a nonempty logical identity")
        references = []
        with self._gc_lock():
            for path in sorted((self.base / "refs").glob("*.json")):
                reference = _json(path)
                self._validate_ref(path, reference)
                if reference["lane"] == lane:
                    references.append(reference)
        return sorted(references, key=lambda reference: reference["consumer"])

    def restore_refs(self, lane: str, *, session: Session) -> list[Handle]:
        """Validate every lane receipt but hash each distinct action only once."""
        with self._gc_lock():
            keys = set()
            for path in sorted((self.base / "refs").glob("*.json")):
                reference = _json(path)
                self._validate_ref(path, reference)
                if reference["lane"] == lane:
                    keys.add(reference["action_key"])
            handles = []
            cache = {}
            for key in sorted(keys):
                manifest = self._verify_tree_locked(key, cache)
                self._pin_locked(session, key)
                handles.append(self._handle(manifest, True))
            return handles

    def _validate_ref(self, path: Path, reference: dict) -> None:
        lane, consumer = reference.get("lane"), reference.get("consumer")
        if (
            set(reference) != {"schema", "lane", "consumer", "action_key"}
            or reference.get("schema") != SCHEMA
            or not isinstance(lane, str)
            or not lane
            or not isinstance(consumer, str)
            or not consumer
        ):
            raise CorruptStore(f"invalid durable reference: {path}")
        if self._ref_path(lane, consumer) != path:
            raise CorruptStore(f"durable reference identity/path mismatch: {path}")
        _key(reference["action_key"])

    def status(self) -> dict:
        """Filesystem accounting only; status does not certify content integrity."""
        report = {
            "schema": SCHEMA,
            "root": str(self.root),
            "verified": False,
            "roles": {},
            "logical_bytes": 0,
            "allocated_bytes": 0,
            "unique_inodes": 0,
        }
        seen = set()
        with self._gc_lock():
            infrastructure = sum(path.lstat().st_blocks * 512 for path in (self.root, self.base))
            report["roles"]["infrastructure"] = {
                "files": 0,
                "logical_bytes": 0,
                "allocated_bytes": infrastructure,
            }
            report["allocated_bytes"] = infrastructure
            seen.update((path.stat().st_dev, path.stat().st_ino) for path in (self.root, self.base))
            for role in ("blobs", "actions", "staging", "leases", "refs", "quarantine", "locks"):
                values = {"files": 0, "logical_bytes": 0, "allocated_bytes": 0}
                root = self.base / role
                info = root.lstat()
                seen.add((info.st_dev, info.st_ino))
                values["allocated_bytes"] += info.st_blocks * 512
                for current, directories, files in os.walk(root, followlinks=False):
                    for name in directories + files:
                        path = Path(current) / name
                        info = path.lstat()
                        scratch_link = role in {"staging", "quarantine"} and stat.S_ISLNK(
                            info.st_mode
                        )
                        if not (
                            stat.S_ISREG(info.st_mode) or stat.S_ISDIR(info.st_mode) or scratch_link
                        ):
                            raise CorruptStore(f"non-regular accounting object: {path}")
                        inode = (info.st_dev, info.st_ino)
                        if inode in seen:
                            continue
                        seen.add(inode)
                        values["allocated_bytes"] += info.st_blocks * 512
                        if stat.S_ISREG(info.st_mode) or scratch_link:
                            values["files"] += 1
                            values["logical_bytes"] += info.st_size
                report["roles"][role] = values
                report["logical_bytes"] += values["logical_bytes"]
                report["allocated_bytes"] += values["allocated_bytes"]
            report["unique_inodes"] = len(seen)
        return report

    def gc(
        self,
        *,
        apply: bool = False,
        grace_seconds: float = 3600,
        ref_alive: Callable[[dict], bool] | None = None,
    ) -> dict:
        """Collect what no live lease, durable ref or young action roots.

        `ref_alive` lets the caller declare a durable ref DEAD: its consumer is gone (the
        lane's worktree was removed, or the Cargo output directory it was bound for no
        longer exists). The store cannot know that itself, and without it a ref outlives
        its consumer forever: on 2026-10-16, 4,726 of 26,588 refs belonged to a worktree
        removed two days earlier, and 2,971 more to build-script directories cargo had
        replaced. A dead ref is dropped only once it is older than the grace period.

        Blobs are hashed only when they are about to be deleted. Every KEPT blob is
        re-verified where it is consumed (`_verify_tree_locked` on bind and restore), and
        hashing all of them here held the GC lock for 3.5 minutes over 8.8 G, stalling
        every lane's binds for that long.
        """
        if not math.isfinite(grace_seconds) or grace_seconds < 0:
            raise StoreError("GC grace must be finite and nonnegative")
        now = time.time()
        report: dict = {
            "apply": apply,
            "actions": [],
            "blobs": [],
            "staging": [],
            "stale_leases": [],
            "dead_refs": [],
            "protected_actions": [],
            "lease_anomalies": [],
        }
        with self._gc_lock():
            roots = set()
            for path in sorted((self.base / "leases").glob("*.json")):
                lease = _json(path)
                self._validate_lease(lease)
                if lease.get("schema") != SCHEMA or self._lease_path(lease.get("id", "")) != path:
                    raise CorruptStore(f"invalid lease: {path}")
                held = self._lock_held(self._lease_lock(lease["id"]))
                alive = _owner_alive(lease["owner"])
                if held or alive:
                    roots.update(_key(key) for key in lease["actions"])
                    if held != alive:
                        report["lease_anomalies"].append(
                            {"id": lease["id"], "held_lock": held, "same_owner_alive": alive}
                        )
                else:
                    report["stale_leases"].append(str(path))
            manifests = {}
            for path in sorted((self.base / "actions").glob("*.json")):
                key = _key(path.stem)
                manifests[key] = self._manifest_locked(key, verify_blobs=False)
                if now - path.stat().st_mtime < grace_seconds:
                    roots.add(key)
            for path in sorted((self.base / "refs").glob("*.json")):
                reference = _json(path)
                self._validate_ref(path, reference)
                if (
                    ref_alive is not None
                    and now - path.stat().st_mtime >= grace_seconds
                    and not ref_alive(reference)
                ):
                    report["dead_refs"].append(str(path))
                    continue
                # Durable missing data is corruption; existence is checked below with the
                # rest of the protected set, content where it is consumed. Every indexed
                # action was read just above; this only catches a ref to a missing one.
                if reference["action_key"] not in manifests:
                    self._manifest_locked(reference["action_key"], verify_blobs=False)
                roots.add(reference["action_key"])
            for manifest in manifests.values():
                if any(
                    dependency not in manifests for dependency in manifest["dependency_actions"]
                ):
                    raise CorruptStore("indexed manifest references a missing dependency action")
            protected = set()

            def visit(key: str) -> None:
                if key in protected:
                    return
                protected.add(key)
                manifest = manifests.get(key)
                if manifest is not None:
                    for dependency in manifest["dependency_actions"]:
                        if dependency not in manifests:
                            raise CorruptStore(f"missing dependency action {dependency}")
                        visit(dependency)

            for key in roots:
                visit(key)
            report["protected_actions"] = sorted(protected)
            for key in protected:
                for output in manifests.get(key, {"outputs": []})["outputs"]:
                    if not _exists(self._blob(output["sha256"])):
                        raise CorruptStore(f"protected action {key} is missing a blob")
            keep_blobs = {
                output["sha256"]
                for key in protected
                if key in manifests
                for output in manifests[key]["outputs"]
            }
            report["actions"] = [
                str(self._action_path(key)) for key in sorted(set(manifests) - protected)
            ]
            for prefix in sorted((self.base / "blobs/sha256").iterdir()):
                if not stat.S_ISDIR(prefix.lstat().st_mode):
                    raise CorruptStore(f"invalid blob prefix: {prefix}")
                for path in sorted(prefix.iterdir()):
                    if path != self._blob(path.name):
                        raise CorruptStore(f"invalid blob path: {path}")
                    if path.name in keep_blobs or now - path.stat().st_mtime < grace_seconds:
                        continue
                    digest, _size = _digest_file(path)
                    if digest != path.name:
                        raise CorruptStore(f"orphan/indexed blob digest mismatch: {path}")
                    report["blobs"].append(str(path))
            for stage in sorted((self.base / "staging").iterdir()):
                if not stat.S_ISDIR(stage.lstat().st_mode):
                    raise CorruptStore(f"non-directory staging object: {stage}")
                producer_path = stage / "producer.json"
                if not _exists(producer_path):
                    continue  # constructor crash before owner record: conservative retention
                producer = _json(producer_path)
                if producer.get("schema") != SCHEMA:
                    raise CorruptStore(f"invalid staging producer: {stage}")
                if self._lock_held(self._action_lock(producer["action_key"])):
                    continue
                if _owner_alive(producer["owner"]) and not _exists(stage / "failure.json"):
                    continue
                if now - stage.stat().st_mtime >= grace_seconds:
                    report["staging"].append(str(stage))
            if apply:
                # Dead refs first: a crash after this leaves only unrooted actions, which
                # the next run collects.
                for name in report["dead_refs"]:
                    Path(name).unlink(missing_ok=True)
                if report["dead_refs"]:
                    _fsync_dir(self.base / "refs")
                for name in report["actions"]:
                    Path(name).unlink()
                # Make index removals durable BEFORE unlinking their blobs.
                _fsync_dir(self.base / "actions")
                for name in report["blobs"] + report["stale_leases"]:
                    Path(name).unlink()
                for name in report["staging"]:
                    _remove_private_tree(Path(name))
                for directory in (
                    self.base / "actions",
                    self.base / "leases",
                    self.base / "staging",
                    *(self.base / "blobs/sha256").iterdir(),
                ):
                    _fsync_dir(directory)
        return report
