from pydantic_settings import BaseSettings


class BotConfig(BaseSettings):
    bot_token: str = ""
    api_base_url: str = "http://localhost:8080/v1"
    api_key: str = ""
    debug: bool = False

    model_config = {"env_prefix": "", "env_file": ".env", "extra": "ignore"}
