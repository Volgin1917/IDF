import httpx
from typing import Any


class ApiClient:
    def __init__(self, base_url: str, api_key: str):
        self.client = httpx.AsyncClient(
            base_url=base_url.rstrip("/"),
            headers={"Authorization": f"Bearer {api_key}"},
            timeout=30,
        )

    async def search_players(self, query: str, limit: int = 10) -> list[dict]:
        resp = await self.client.get("/players/search", params={"q": query, "limit": limit})
        resp.raise_for_status()
        return resp.json().get("data", [])

    async def get_player(self, player_id: int) -> dict | None:
        resp = await self.client.get(f"/players/{player_id}")
        if resp.status_code == 404:
            return None
        resp.raise_for_status()
        return resp.json().get("data")

    async def get_player_stats(self, player_id: int, season: str | None = None) -> list[dict]:
        params = {}
        if season:
            params["season"] = season
        resp = await self.client.get(f"/players/{player_id}/stats", params=params)
        resp.raise_for_status()
        data = resp.json().get("data", {})
        return data.get("seasons", [])

    async def get_ai_analysis(self, player_id: int, analysis_type: str = "full") -> dict:
        resp = await self.client.get(
            f"/players/{player_id}/ai-analysis",
            params={"type": analysis_type},
        )
        resp.raise_for_status()
        return resp.json()

    async def compare_players(self, player_ids: list[int], season: str | None = None) -> dict:
        body = {"player_ids": player_ids}
        if season:
            body["season"] = season
        resp = await self.client.post("/players/compare", json=body)
        resp.raise_for_status()
        return resp.json().get("data", {})

    async def get_news(self, limit: int = 10) -> list[dict]:
        resp = await self.client.get("/news", params={"limit": limit})
        resp.raise_for_status()
        return resp.json().get("data", [])

    async def close(self):
        await self.client.aclose()
