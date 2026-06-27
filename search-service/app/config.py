import logging
import sys

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    database_url: str = "postgresql://scrinium:scrinium@localhost:5434/search"
    kafka_brokers: str = "localhost:9092"
    kafka_group_id: str = "search-service"
    http_host: str = "127.0.0.1"
    http_port: int = 8092
    log_level: str = "INFO"

    model_config = {"env_prefix": "SEARCH_"}


settings = Settings()


def setup_logging() -> None:
    logging.basicConfig(
        level=settings.log_level.upper(),
        format="%(asctime)s %(levelname)-5s [%(name)s] %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        stream=sys.stdout,
    )


setup_logging()
