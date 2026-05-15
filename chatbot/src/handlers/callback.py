from aiogram import Router, types

from api_client import ApiClient
from keyboards import player_actions_keyboard

router = Router()


@router.callback_query(lambda c: c.data.startswith("player:"))
async def cb_player(callback: types.CallbackQuery, api: ApiClient):
    player_id = int(callback.data.split(":")[1])
    player = await api.get_player(player_id)
    if not player:
        await callback.message.edit_text(f"Player {player_id} not found")
        return

    text = (
        f"<b>{player['full_name']}</b>\n"
        f"Position: {player['position']}\n"
        f"Team: {player.get('current_team_abbreviation') or 'FA'}\n"
    )
    await callback.message.edit_text(text, reply_markup=player_actions_keyboard(player_id))
    await callback.answer()


@router.callback_query(lambda c: c.data.startswith("stats:"))
async def cb_stats(callback: types.CallbackQuery, api: ApiClient):
    player_id = int(callback.data.split(":")[1])
    seasons = await api.get_player_stats(player_id)

    if not seasons:
        await callback.message.edit_text("No stats available")
        await callback.answer()
        return

    lines = [f"<b>Stats for player {player_id}</b>"]
    for s in seasons[:5]:
        lines.append(
            f"{s['season']} — {s.get('team') or 'N/A'}: "
            f"{s['games_played']}GP {s['goals']}G {s['assists']}A {s['points']}P"
        )

    await callback.message.edit_text("\n".join(lines))
    await callback.answer()


@router.callback_query(lambda c: c.data.startswith("analyze:"))
async def cb_analyze(callback: types.CallbackQuery, api: ApiClient):
    player_id = int(callback.data.split(":")[1])
    await callback.message.edit_text("Running AI analysis...")
    await callback.answer()

    try:
        result = await api.get_ai_analysis(player_id, "full")
    except Exception as e:
        await callback.message.edit_text(f"Analysis failed: {e}")
        return

    analysis = result.get("analysis", {})
    summary = analysis.get("summary") or "No summary"
    strengths = analysis.get("strengths") or []
    weaknesses = analysis.get("weaknesses") or []

    text = f"<b>AI Analysis</b>\n\n{summary}\n\n"
    if strengths:
        text += "<b>Strengths:</b>\n" + "\n".join(f"  • {s}" for s in strengths[:3]) + "\n\n"
    if weaknesses:
        text += "<b>Weaknesses:</b>\n" + "\n".join(f"  • {w}" for w in weaknesses[:3])

    await callback.message.edit_text(text)


@router.callback_query(lambda c: c.data == "latest_news")
async def cb_latest_news(callback: types.CallbackQuery, api: ApiClient):
    await callback.answer("Fetching news...")
    try:
        news_items = await api.get_news(limit=5)
    except Exception as e:
        await callback.message.answer(f"Error: {e}")
        return

    if not news_items:
        await callback.message.answer("No news available")
        return

    lines = ["<b>Latest NHL News</b>\n"]
    for item in news_items:
        icon = {"positive": "🟢", "negative": "🔴", "neutral": "⚪"}.get(
            item.get("sentiment", ""), "⚪"
        )
        lines.append(f"{icon} {item.get('title', '?')}")

    await callback.message.answer("\n".join(lines))


@router.callback_query(lambda c: c.data == "help")
async def cb_help(callback: types.CallbackQuery):
    await callback.answer()
    await callback.message.answer(
        "🏒 ICE DATA FORGE Bot\n\n"
        "/search <name> — find a player\n"
        "/player <id> — player profile\n"
        "/stats <id> [season] — stats\n"
        "/analyze <id> — AI report\n"
        "/compare <id1> <id2> — compare\n"
        "/news — latest news\n"
        "/help — this message"
    )


@router.callback_query(lambda c: c.data == "cancel")
async def cb_cancel(callback: types.CallbackQuery):
    await callback.message.edit_text("Cancelled")
    await callback.answer()

