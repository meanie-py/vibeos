"""Crash-safe, process-safe reservation ledger for paid rustconv attempts.

The JSON file is a human-readable durable snapshot. An unsettled reservation is
never released automatically: callers must settle with a known actual amount.
"""

from __future__ import annotations

import json
import os
import tempfile
import threading
from dataclasses import dataclass
from decimal import Decimal, InvalidOperation
from pathlib import Path
from typing import Any


class BudgetError(RuntimeError):
    """Base class for budget ledger errors."""


class BudgetStateError(BudgetError):
    """The persisted ledger is corrupt, inconsistent, or uses another cap."""


class BudgetExceeded(BudgetError):
    """A reservation cannot fit under the configured cap."""


class BudgetOverrun(BudgetError):
    """Actual usage exceeded its reservation; the overrun was durably recorded."""


@dataclass(frozen=True)
class Reservation:
    attempt_id: str
    upper_bound_usd: Decimal


_LOCAL_LOCKS_GUARD = threading.Lock()
_LOCAL_LOCKS: dict[str, threading.RLock] = {}


def _decimal(value: Decimal | str, what: str) -> Decimal:
    if isinstance(value, bool):
        raise ValueError(f"{what} must be a finite nonnegative decimal")
    try:
        number = value if isinstance(value, Decimal) else Decimal(str(value))
    except (InvalidOperation, ValueError, TypeError) as exc:
        raise ValueError(f"{what} must be a finite nonnegative decimal") from exc
    if not number.is_finite() or number < 0:
        raise ValueError(f"{what} must be a finite nonnegative decimal")
    return number


def _amount(value: Any, what: str) -> Decimal:
    if not isinstance(value, str):
        raise BudgetStateError(f"invalid {what} in budget ledger: expected decimal string")
    try:
        return _decimal(value, what)
    except ValueError as exc:
        raise BudgetStateError(f"invalid {what} in budget ledger") from exc


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key in budget ledger: {key!r}")
        result[key] = value
    return result


def _fmt(value: Decimal) -> str:
    # Decimal's fixed-point string avoids exponent notation in the snapshot.
    return format(value, "f")


class Budget:
    """Persistent budget with atomic reservations and conservative crash recovery.

    ``path`` names the JSON snapshot. A sibling ``<name>.lock`` serializes all
    processes; temporary snapshots are fsynced and atomically renamed.
    """

    _VERSION = 1
    _FIELDS = frozenset(
        {"version", "limit_usd", "spent_usd", "debt_usd", "reservations", "settled"}
    )

    def __init__(self, path: Path, limit_usd: Decimal | str):
        self.path = Path(path).resolve()
        self.limit_usd = _decimal(limit_usd, "limit_usd")
        self.lock_path = self.path.with_name(self.path.name + ".lock")
        self.path.parent.mkdir(parents=True, exist_ok=True)
        key = str(self.lock_path.resolve())
        with _LOCAL_LOCKS_GUARD:
            self._thread_lock = _LOCAL_LOCKS.setdefault(key, threading.RLock())
        with self._locked():
            if self.path.exists():
                self._read()
            else:
                self._write(self._empty_state())

    def reserve(self, attempt_id: str, upper_bound_usd: Decimal | str) -> Reservation:
        if not isinstance(attempt_id, str) or not attempt_id.strip():
            raise ValueError("attempt_id must be a nonempty string")
        amount = _decimal(upper_bound_usd, "upper_bound_usd")
        with self._locked():
            state = self._read()
            if state["debt_usd"] > 0:
                raise BudgetExceeded(
                    f"budget is blocked by recorded overrun debt of ${_fmt(state['debt_usd'])}"
                )
            if attempt_id in state["reservations"] or attempt_id in state["settled"]:
                raise BudgetStateError(f"attempt_id already used: {attempt_id!r}")
            committed = state["spent_usd"] + sum(state["reservations"].values(), Decimal(0))
            if committed + amount > self.limit_usd:
                available = max(Decimal(0), self.limit_usd - committed)
                raise BudgetExceeded(
                    f"reservation ${_fmt(amount)} exceeds available budget ${_fmt(available)}"
                )
            state["reservations"][attempt_id] = amount
            self._write(state)
        return Reservation(attempt_id=attempt_id, upper_bound_usd=amount)

    def settle(self, attempt_id: str, actual_usd: Decimal | str | None) -> None:
        """Settle a reserved attempt; ``None`` deliberately keeps it reserved."""
        if not isinstance(attempt_id, str) or not attempt_id.strip():
            raise ValueError("attempt_id must be a nonempty string")
        if actual_usd is None:
            # No write is required: preserving the reservation is the safe state.
            with self._locked():
                state = self._read()
                if attempt_id not in state["reservations"]:
                    raise BudgetStateError(f"no unsettled reservation for {attempt_id!r}")
            return None
        actual = _decimal(actual_usd, "actual_usd")
        overrun = False
        with self._locked():
            state = self._read()
            if attempt_id not in state["reservations"]:
                raise BudgetStateError(f"no unsettled reservation for {attempt_id!r}")
            reserved = state["reservations"].pop(attempt_id)
            state["spent_usd"] += actual
            excess = max(Decimal(0), actual - reserved)
            state["debt_usd"] += excess
            state["settled"][attempt_id] = {"reserved_usd": reserved, "actual_usd": actual}
            # Persist the actual charge and debt before reporting an overrun.
            self._write(state)
            overrun = excess > 0
        if overrun:
            raise BudgetOverrun(
                f"attempt {attempt_id!r} actual ${_fmt(actual)} exceeded reservation "
                f"${_fmt(reserved)}; ${_fmt(excess)} debt recorded and future reservations blocked"
            )
        return None

    def snapshot(self) -> dict[str, Any]:
        """Return a fresh, JSON-compatible, human-readable ledger snapshot."""
        with self._locked():
            state = self._read()
            return self._serialize(state)

    def _empty_state(self) -> dict[str, Any]:
        return {
            "version": self._VERSION,
            "limit_usd": self.limit_usd,
            "spent_usd": Decimal(0),
            "debt_usd": Decimal(0),
            "reservations": {},
            "settled": {},
        }

    def _read(self) -> dict[str, Any]:
        try:
            with self.path.open("r", encoding="utf-8") as stream:
                raw = json.load(stream, object_pairs_hook=_unique_object)
        except (OSError, json.JSONDecodeError, ValueError) as exc:
            raise BudgetStateError(f"cannot read budget ledger {self.path}: {exc}") from exc
        if not isinstance(raw, dict) or set(raw) != self._FIELDS:
            raise BudgetStateError("budget ledger has unexpected fields or shape")
        if raw["version"] != self._VERSION:
            raise BudgetStateError(f"unsupported budget ledger version: {raw['version']!r}")
        cap = _amount(raw["limit_usd"], "limit_usd")
        if cap != self.limit_usd:
            raise BudgetStateError(
                f"budget cap changed: ledger has ${_fmt(cap)}, "
                f"caller requested ${_fmt(self.limit_usd)}"
            )
        spent = _amount(raw["spent_usd"], "spent_usd")
        debt = _amount(raw["debt_usd"], "debt_usd")
        if not isinstance(raw["reservations"], dict) or not isinstance(raw["settled"], dict):
            raise BudgetStateError("budget ledger reservations/settled must be objects")
        reservations: dict[str, Decimal] = {}
        for key, value in raw["reservations"].items():
            if not isinstance(key, str) or not key.strip():
                raise BudgetStateError("budget ledger contains an invalid reservation id")
            reservations[key] = _amount(value, f"reservation {key}")
        settled: dict[str, dict[str, Decimal]] = {}
        for key, value in raw["settled"].items():
            if (
                not isinstance(key, str)
                or not key.strip()
                or not isinstance(value, dict)
                or set(value) != {"reserved_usd", "actual_usd"}
            ):
                raise BudgetStateError("budget ledger contains an invalid settled attempt")
            settled[key] = {
                "reserved_usd": _amount(value["reserved_usd"], f"settled reservation {key}"),
                "actual_usd": _amount(value["actual_usd"], f"settled actual {key}"),
            }
        if set(reservations).intersection(settled):
            raise BudgetStateError("attempt appears in both reservations and settled records")
        settled_spent = sum((item["actual_usd"] for item in settled.values()), Decimal(0))
        settled_debt = sum(
            (
                max(Decimal(0), item["actual_usd"] - item["reserved_usd"])
                for item in settled.values()
            ),
            Decimal(0),
        )
        if spent != settled_spent or debt != settled_debt:
            raise BudgetStateError("budget ledger totals disagree with settled attempt records")
        if spent + sum(reservations.values(), Decimal(0)) > self.limit_usd and debt == 0:
            raise BudgetStateError("budget ledger exceeds cap without recorded debt")
        return {
            "version": self._VERSION,
            "limit_usd": cap,
            "spent_usd": spent,
            "debt_usd": debt,
            "reservations": reservations,
            "settled": settled,
        }

    def _serialize(self, state: dict[str, Any]) -> dict[str, Any]:
        return {
            "version": self._VERSION,
            "limit_usd": _fmt(state["limit_usd"]),
            "spent_usd": _fmt(state["spent_usd"]),
            "debt_usd": _fmt(state["debt_usd"]),
            "reservations": {
                key: _fmt(value) for key, value in sorted(state["reservations"].items())
            },
            "settled": {
                key: {
                    "reserved_usd": _fmt(value["reserved_usd"]),
                    "actual_usd": _fmt(value["actual_usd"]),
                }
                for key, value in sorted(state["settled"].items())
            },
        }

    def _write(self, state: dict[str, Any]) -> None:
        payload = json.dumps(self._serialize(state), indent=2, sort_keys=True) + "\n"
        fd, temp_name = tempfile.mkstemp(
            prefix=f".{self.path.name}.", suffix=".tmp", dir=self.path.parent
        )
        try:
            with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as stream:
                stream.write(payload)
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(temp_name, self.path)
            # Persist the rename itself on platforms with directory fsync.
            if os.name != "nt":
                dir_fd = os.open(self.path.parent, os.O_RDONLY)
                try:
                    os.fsync(dir_fd)
                finally:
                    os.close(dir_fd)
        finally:
            try:
                os.unlink(temp_name)
            except FileNotFoundError:
                pass

    class _Lock:
        def __init__(self, owner: Budget):
            self.owner = owner
            self.stream = None

        def __enter__(self):
            self.owner._thread_lock.acquire()
            try:
                self.stream = self.owner.lock_path.open("a+b")
                if os.name == "nt":
                    import msvcrt

                    self.stream.seek(0)
                    # Ensure a byte exists for the byte-range lock.
                    if self.stream.read(1) == b"":
                        self.stream.seek(0)
                        self.stream.write(b"0")
                        self.stream.flush()
                    self.stream.seek(0)
                    msvcrt.locking(self.stream.fileno(), msvcrt.LK_LOCK, 1)
                else:
                    import fcntl

                    fcntl.flock(self.stream.fileno(), fcntl.LOCK_EX)
                return self
            except BaseException:
                if self.stream is not None:
                    self.stream.close()
                self.owner._thread_lock.release()
                raise

        def __exit__(self, exc_type, exc, tb):
            if self.stream is None:
                raise BudgetStateError("Budget lock was not acquired")
            try:
                if os.name == "nt":
                    import msvcrt

                    self.stream.seek(0)
                    msvcrt.locking(self.stream.fileno(), msvcrt.LK_UNLCK, 1)
                else:
                    import fcntl

                    fcntl.flock(self.stream.fileno(), fcntl.LOCK_UN)
                self.stream.close()
            finally:
                self.owner._thread_lock.release()

    def _locked(self) -> Budget._Lock:
        return self._Lock(self)
