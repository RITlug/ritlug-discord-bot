# ritlug-discord-bot
A discord bot written in Rust for helping out with the RITlug discord server

## Configuration

The Discord bot token is either read from the environment variable `BOT_TOKEN`, either from a .env file in the current directory or passed as an environment variable directly.

### Example `config.json`

`irc.channels` is a map from IRC channel names to Discord channel IDs. `irc.use_tls` defaults to `true`. If `avatar` is omitted or set to the empty string, the default Discord avatar will be used.

```json
{
    "irc": {
        "server": "example.com",
        "nickname": "Example",
        "use_tls": true,
        "avatar": "",
        "channels": {
            "#channel-1": 123456789012345678,
            "#channel-2": 628318530717957646
        }
    }
}
```
