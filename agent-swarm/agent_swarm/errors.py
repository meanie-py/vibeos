"""Exception hierarchy for the agent-swarm platform."""

from __future__ import annotations


class SwarmError(Exception):
    """Base for all agent-swarm errors."""


class ConfigError(SwarmError):
    """Missing or malformed configuration."""


class HttpError(SwarmError):
    """A non-2xx HTTP response from a backend service."""

    def __init__(self, status: int, url: str, body: str = ""):
        self.status = status
        self.url = url
        self.body = body
        super().__init__(f"HTTP {status} for {url}: {body[:300]}")


class NotFound(SwarmError):
    """A requested entity does not exist."""
