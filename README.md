# brioschenbot-rs

This is the BrioschenBot Telegram bot.

# Deployment

- Build the docker container
  - `docker build -t brioschenbot-rs .`
- Run the docker container
  - `docker run --name brioschenbot brioschenbot-rs`
  - Change all the environment variables to your liking as follows
```
BOT_TOKEN=
DB_IP=
DB_NAME=
DB_USER=
DB_PASS=
TG_LOG_CHATID=
TS_IP=
TS_QUERY_PORT=
TS_SERVER_PORT=
TS_USER=
TS_PASSWORD=
SURPRISE_TARGET=
LOG_RANDNUM_EXCEPTION=
```