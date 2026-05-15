from aiogram import Router, types
from aiogram.filters import Command

from api_client import ApiClient
from keyboards import player_actions_keyboard, season_selector_keyboard

router = Router()


@router.message(Command("player"))
async def cmd_player(message: types.Message, api: ApiClient):
    args = message.text.split(maxsplit=1)
    if len(args) < 2:
        await message.answer("Usage: /player <id>")
        return

    try:
        player_id = int(args[1].strip())
    except ValueError:
        await message.answer("Player ID must be a number")
        return

    try:
        player = await api.get_player(player_id)
    except Exception as e:
        await message.answer(f"Error: {e}")
        return

    if not player:
        await message.answer(f"Player {player_id} not found")
        return

    text = (
        f"<b>{player['full_name']}</b>\n"
        f"Position: {player['position']}\n"
        f"Team: {player.get('current_team_abbreviation') or 'FA'}\n"
        f"Height: {player.get('height_cm') or 'N/A'} cm\n"
        f"Weight: {player.get('weight_lbs') or 'N/A'} lbs\n"
        f"Born: {player.get('birth_date') or 'N/A'}\n"
        f"Active: {'Yes' if player.get('is_active') else 'No'}"
    )

    await message.answer(text, reply_markup=player_actions_keyboard(player_id))


@router.message(Command("stats"))
async def cmd_stats(message: types.Message, api: ApiClient):
    args = message.text.split()
    if len(args) < 2:
        await message.answer("Usage: /stats <id> [season]")
        return

    try:
        player_id = int(args[1])
    except ValueError:
        await message.answer("Player ID must be a number")
        return

    season = args[2] if len(args) > 2 else None

    try:
        seasons = await api.get_player_stats(player_id, season)
    except Exception as e:
        await message.answer(f"Error: {e}")
        return

    if not seasons:
        await message.answer(f"No stats found for player {player_id}")
        return

    if season:
        lines = [f"<b>Stats for {season}:</b>"]
    else:
        lines = ["<b>Season stats:</b>"]

    for s in seasons[:5]:
        lines.append(
            f"{s['season']} — {s.get('team') or 'N/A'}: "
            f"{s['games_played']}GP {s['goals']}G {s['assists']}A {s['points']}P "
            f"{s['plus_minus']:+d}"
        )

    await message.answer("\n".join(lines))


@router.message(Command("analyze"))
async def cmd_analyze(message: types.Message, api: ApiClient):
    args = message.text.split(maxsplit=1)
    if len(args) < 2:
        await message.answer("Usage: /analyze <id>")
        return

    try:
        player_id = int(args[1].strip())
    except ValueError:
        await message.answer("Player ID must be a number")
        return

    await message.answer("Running AI analysis... (this may take 10-15s)")

    try:
        result = await api.get_ai_analysis(player_id, "full")
    except Exception as e:
        await message.answer(f"AI analysis failed: {e}")
        return

    analysis = result.get("analysis", {})
    summary = analysis.get("summary") or "No summary available"
    strengths = analysis.get("strengths") or []
    weaknesses = analysis.get("weaknesses") or []
    confidence = analysis.get("confidence_score") or 0

    text = (
        f"<b>AI Analysis</b>\n\n"
        f"{summary}\n\n"
    )

    if strengths:
        text += "<b>Strengths:</b>\n"
        for s in strengths[:3]:
            text += f"  • {s}\n"
        text += "\n"

    if weaknesses:
        text += "<b>Areas to improve:</b>\n"
        for w in weaknesses[:3]:
            text += f"  • {w}\n"
        text += "\n"

    text += f"Confidence: {float(confidence) * 100:.0f}% | Model: GPT-4"

    await message.answer(text)

