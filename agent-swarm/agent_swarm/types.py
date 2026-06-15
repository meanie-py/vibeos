"""Shared types — enums and dataclasses passed across the core, instead of bare
dicts/tuples/magic-strings.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum, IntEnum


class Priority(str, Enum):
    """Delivery priority of a message. (str-Enum so it serializes as 'high'/'low'.)"""

    HIGH = "high"
    LOW = "low"


class VikunjaPriority(IntEnum):
    """Vikunja's 0–5 priority scale, with display labels and a safe parser."""

    UNSET = 0
    LOW = 1
    MEDIUM = 2
    HIGH = 3
    URGENT = 4
    DO_NOW = 5

    @property
    def label(self) -> str:
        return {0: "-", 1: "low", 2: "medium", 3: "High", 4: "Urgent", 5: "DO NOW"}[self.value]

    @classmethod
    def parse(cls, value: object) -> VikunjaPriority:
        try:
            return cls(int(value))  # type: ignore[arg-type]
        except (ValueError, TypeError):
            return cls.UNSET


@dataclass(frozen=True)
class Post:
    """A Mattermost post, normalized. `user` is the resolved @handle (webhook
    override_username, or the user_id resolved to a username)."""

    id: str
    create_at: int
    user: str
    user_id: str
    message: str


@dataclass(frozen=True)
class HighMessage:
    """A high-priority message to deliver. Serializes with `from` (a Python keyword,
    so the field is `sender`) for the channel server's `<channel from=...>` meta."""

    sender: str
    message: str
    post_id: str
    channel: str

    def to_dict(self) -> dict:
        return {
            "from": self.sender,
            "message": self.message,
            "post_id": self.post_id,
            "channel": self.channel,
        }


@dataclass
class DrainResult:
    """Result of a delivery drain: high-priority messages + a count of the rest."""

    high: list[HighMessage] = field(default_factory=list)
    low_count: int = 0

    def to_dict(self) -> dict:
        return {"high": [h.to_dict() for h in self.high], "low_count": self.low_count}


class AgentStatus(str, Enum):
    ONLINE = "online"
    STALE = "stale"
    OFFLINE = "offline"


@dataclass
class Agent:
    """A roster entry (who is in the swarm, on what, live-or-not)."""

    identity: str
    status: AgentStatus = AgentStatus.OFFLINE
    lead: str | None = None
    session_id: str | None = None
    machine: str | None = None
    pid: int | None = None
    project: str | None = None
    quest: str | None = None
    branch: str | None = None
    started_at: float | None = None
    last_seen: float | None = None
    age_secs: float = 0.0

    def to_dict(self) -> dict:
        d = self.__dict__.copy()
        d["status"] = self.status.value
        return d
