# Contributing to ritwug-discord-bot

For the most part, try to follow the [Rust Style Guide](https://doc.rust-lang.org/beta/style-guide/index.html). 

If your contribution adds a new command, give it its own file in `src/commands`. If your contribution adds a significant new feature, give it its own module (like the IRC bridge in `src/irc_bridge` and the SMTP api in `src/smtp`). Ensure that your code is clean and readable so that others can read and add to it.

If you need to add new dependencies, try using [feature flags](https://doc.rust-lang.org/cargo/reference/features.html) to only include the features you need (thus minimizing the number of new dependencies that they in turn bring in)

Test your new features locally by [creating a Discord bot](https://discord.com/developers/docs/getting-started) and following the instructions in [README.md](README.md) for setting the token in an environment variable. A `config.json` containing just `{}` is valid and will disable most bot features.

Once your new features are complete and fully tested, [create a pull request](https://github.com/RITlug/ritlug-discord-bot/pulls) against the main repo.
