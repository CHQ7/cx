"""Python client for ga-core HTTP API.

This client allows Python frontends (TUI, Streamlit, etc.) to communicate
with the Rust core via HTTP.
"""

import requests
import json
from typing import Optional, Dict, Any, Iterator


class GaCoreClient:
    """Client for the ga-core HTTP API."""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url.rstrip("/")

    def health(self) -> str:
        """Check if the server is healthy."""
        resp = requests.get(f"{self.base_url}/health")
        resp.raise_for_status()
        return resp.text

    def run(
        self,
        system_prompt: str,
        user_input: str,
        max_turns: int = 40,
        verbose: bool = True,
    ) -> Dict[str, Any]:
        """Run an agent task.

        Returns a dict with keys: result, data, turns
        """
        resp = requests.post(
            f"{self.base_url}/api/agent/run",
            json={
                "system_prompt": system_prompt,
                "user_input": user_input,
                "max_turns": max_turns,
                "verbose": verbose,
            },
        )
        resp.raise_for_status()
        return resp.json()

    def get_schema(self) -> list:
        """Get the tool schema."""
        resp = requests.get(f"{self.base_url}/api/schema")
        resp.raise_for_status()
        return resp.json()

    def run_stream(
        self,
        system_prompt: str,
        user_input: str,
        max_turns: int = 40,
    ) -> Iterator[Dict[str, Any]]:
        """Run an agent task with streaming output.

        Yields dicts with partial results.
        """
        # TODO: Implement when streaming endpoint is ready
        raise NotImplementedError("Streaming not yet implemented")


# Convenience function for quick usage
def create_client(base_url: str = "http://localhost:8080") -> GaCoreClient:
    """Create a new ga-core client."""
    return GaCoreClient(base_url)
