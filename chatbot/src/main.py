import asyncio
import logging

from aiogram import Bot, Dispatcher, types
from aiogram.client.default import DefaultBotProperties
from aiogram.enums import ParseMode

from .api_client import ApiClient
from .config import BotConfig
from .handlers import start, search, player, compare, news, callback

logging.basicConfig(level=logging.INFO)
log = logging.getLogger(__name__)


async def main():
    cfg = BotConfig()

    if not cfg.bot_token:
        log.error("BOT_TOKEN is not set")
        return

    bot = Bot(
        token=cfg.bot_token,
        default=DefaultBotProperties(parse_mode=ParseMode.HTML),
    )
    api = ApiClient(cfg.api_base_url, cfg.api_key)
    dp = Dispatcher(api=api)

    dp.include_router(start.router)
    dp.include_router(search.router)
    dp.include_router(player.router)
    dp.include_router(compare.router)
    dp.include_router(news.router)
    dp.include_router(callback.router)

    log.info("Starting ICE DATA FORGE bot...")
    await bot.delete_webhook(drop_pending_updates=True)
    await dp.start_polling(bot)


if __name__ == "__main__":
    asyncio.run(main())
