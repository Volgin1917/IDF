from aiogram import Router, types
from aiogram.filters import Command

from api_client import ApiClient

router = Router()


@router.message(Command("news"))
async def cmd_news(message: types.Message, api: ApiClient):
    await message.answer("Fetching latest NHL news...")

    try:
        news_items = await api.get_news(limit=10)
    except Exception as e:
        await message.answer(f"Error fetching news: {e}")
        return

    if not news_items:
        await message.answer("No news available")
        return

    lines = ["<b>Latest NHL News</b>\n"]
    for item in news_items[:10]:
        title = item.get("title", "?")
        source = item.get("source", "?")
        sentiment = item.get("sentiment", "")
        icon = {"positive": "🟢", "negative": "🔴", "neutral": "⚪"}.get(sentiment, "⚪")
        lines.append(f"{icon} {title}")
        lines.append(f"   — {source}\n")

    await message.answer("\n".join(lines))

