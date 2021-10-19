# brioschenbot-rs

This is the BrioschenBot Telegram bot.

# Deployment

- Edit the `configuration.json` file to your liking
- Build the docker container
  - `docker build -t brioschenbot-rs .`
- Run the docker container
  - `docker run --name brioschenbot brioschenbot-rs`