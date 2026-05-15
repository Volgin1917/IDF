from aiogram import Router, types
from aiogram.filters import Command

from api_client import ApiClient

router = Router()


@router.message(Command("compare"))
async def cmd_compare(message: types.Message, api: ApiClient):
    args = message.text.split()
    if len(args) < 3:
        await message.answer("Usage: /compare <id1> <id2> [season]")
        return

    try:
        id1 = int(args[1])
        id2 = int(args[2])
    except ValueError:
        await message.answer("Player IDs must be numbers")
        return

    season = args[3] if len(args) > 3 else None

    await message.answer("Comparing players...")

    try:
        result = await api.compare_players([id1, id2], season)
    except Exception as e:
        await message.answer(f"Compare failed: {e}")
        return

    comparison = result.get("comparison", [])

    lines = ["<b>Player Comparison</b>\n"]
    for entry in comparison:
        p = entry.get("player", {})
        seasons = entry.get("seasons", [])
        lines.append(f"<b>{p.get('full_name', '?')}</b> ({p.get('position', '?')})")
        if seasons:
            s = seasons[0]
            lines.append(f"  {s.get('season', '?')}: {s.get('games_played', 0)}GP "
                         f"{s.get('goals', 0)}G {s.get('assists', 0)}A {s.get('points', 0)}P")
        lines.append("")

    await message.answer("\n".join(lines))

