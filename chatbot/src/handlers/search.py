from aiogram import Router, types
from aiogram.filters import Command

from api_client import ApiClient
from keyboards import player_list_keyboard

router = Router()


@router.message(Command("search"))
async def cmd_search(message: types.Message, api: ApiClient):
    args = message.text.split(maxsplit=1)
    if len(args) < 2:
        await message.answer("Usage: /search <player name>")
        return

    query = args[1].strip()
    if len(query) < 2:
        await message.answer("Query must be at least 2 characters")
        return

    await message.answer(f"Searching for \"{query}\"...")

    try:
        players = await api.search_players(query)
    except Exception as e:
        await message.answer(f"Error: {e}")
        return

    if not players:
        await message.answer(f"No players found for \"{query}\"")
        return

    text = f"Found {len(players)} player(s):\n"
    await message.answer(text, reply_markup=player_list_keyboard(players))

