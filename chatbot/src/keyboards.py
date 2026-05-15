from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton
from aiogram.utils.keyboard import InlineKeyboardBuilder


def player_actions_keyboard(player_id: int) -> InlineKeyboardMarkup:
    b = InlineKeyboardBuilder()
    b.button(text="Stats", callback_data=f"stats:{player_id}")
    b.button(text="Analyze", callback_data=f"analyze:{player_id}")
    b.button(text="News", callback_data=f"news:{player_id}")
    b.adjust(3)
    return b.as_markup()


def player_list_keyboard(players: list[dict]) -> InlineKeyboardMarkup:
    b = InlineKeyboardBuilder()
    for p in players:
        label = f"{p['full_name']} ({p['position']})"
        b.button(text=label, callback_data=f"player:{p['nhl_player_id']}")
    b.adjust(1)
    return b.as_markup()


def season_selector_keyboard(player_id: int, seasons: list[str]) -> InlineKeyboardMarkup:
    b = InlineKeyboardBuilder()
    for s in seasons[:5]:
        b.button(text=s, callback_data=f"season:{player_id}:{s}")
    b.button(text="Cancel", callback_data="cancel")
    b.adjust(2)
    return b.as_markup()


def main_menu_keyboard() -> InlineKeyboardMarkup:
    b = InlineKeyboardBuilder()
    b.button(text="Search Player", switch_inline_query_current_chat="")
    b.button(text="Latest News", callback_data="latest_news")
    b.button(text="Help", callback_data="help")
    b.adjust(1)
    return b.as_markup()
