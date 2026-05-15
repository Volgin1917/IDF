from aiogram import Router, types
from aiogram.filters import Command

from keyboards import main_menu_keyboard

router = Router()


@router.message(Command("start"))
async def cmd_start(message: types.Message):
    await message.answer(
        "🏒 ICE DATA FORGE — NHL Player Analytics\n\n"
        "Commands:\n"
        "/search <name> — search players\n"
        "/player <id> — player info\n"
        "/stats <id> [season] — season stats\n"
        "/analyze <id> — AI-powered analysis\n"
        "/compare <id1> <id2> [season] — compare two players\n"
        "/news — latest hockey news\n"
        "/help — this message",
        reply_markup=main_menu_keyboard(),
    )


@router.message(Command("help"))
async def cmd_help(message: types.Message):
    await cmd_start(message)

