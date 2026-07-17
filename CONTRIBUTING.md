# Contributing to dismcp

First off, thank you for considering contributing to `dismcp`!

The goal of this project is to provide a comprehensive, lightweight, and fast HTTP server that exposes Discord bot functionality via the Model Context Protocol (MCP).

## How to Contribute

1. **Fork the repository** and create your branch from `main`.
2. **Make your changes**. If you've added code that should be tested, add tests.
3. **Ensure the test suite passes** by running the formatting and linting checks.
4. **Commit your changes** following our commit conventions.
5. **Issue a pull request!**

## Development Setup

To get started with development, you'll need the Rust toolchain installed.

1. Clone your fork: `git clone https://github.com/semanavasco/dismcp.git`
2. Enter the directory: `cd dismcp`
3. Run the project: `DISCORD_TOKEN=your_bot_token cargo run`

## Adding New Tools

If you are adding a new Discord tool, follow these steps:

1. **Find the right module:** Look in `src/tools/` for the appropriate category (e.g., `guild.rs`, `message.rs`). If it's a completely new category, create a new file (e.g., `src/tools/my_category.rs`) and expose it in `src/tools/mod.rs` and `src/server.rs`.
2. **Define the Params struct:** Use `schemars::JsonSchema` and `serde::Deserialize` to define the JSON payload your tool accepts. Use the `#[schemars(description = "...")]` macro to thoroughly document every field, as this is what AI agents will read to understand how to use your tool!
3. **Implement the tool:** Add an asynchronous function to the `Server` impl block annotated with `#[tool(description = "...")]`.
4. **Use Serenity:** Use `self.bot_http()` to access the Discord API via the `serenity` crate.
5. **Return standard results:** Map serenity errors to `ErrorData::internal_error` or `ErrorData::invalid_params` where appropriate, and return successful payloads using the `structured()` helper.
6. **Update README:** Don't forget to add your new tool to the table in `README.md`!

## Quick Testing

You can test your tools either natively via `stdio` (the default) or via HTTP. We recommend testing both methods to ensure full compatibility.

### Testing via Stdio

The easiest way to test `stdio` mode interactively is using the official MCP Inspector. This will spin up a web interface where you can visually inspect and trigger your tools:

```bash
DISCORD_TOKEN=your_bot_token npx @modelcontextprotocol/inspector cargo run
```

### Testing via HTTP

To test via HTTP, first start the server in HTTP mode:

```bash
MCP_TRANSPORT=http DISCORD_TOKEN=your_bot_token cargo run
```

Then you can use `curl` to manually send JSON-RPC requests to the server. Here are some examples:

### List tools

```bash
curl -sS http://127.0.0.1:3000 \
  -H 'content-type: application/json' \
  -H 'accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | jq
```

### Call `get_guilds`

```bash
curl -sS http://127.0.0.1:3000 \
  -H 'content-type: application/json' \
  -H 'accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_guilds","arguments":{}}}' | jq '.result.structuredContent'
```

### Call `get_guild` with a guild ID

```bash
curl -sS http://127.0.0.1:3000 \
  -H 'content-type: application/json' \
  -H 'accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_guild","arguments":{"guild_id":"1234567890"}}}' | jq '.result.structuredContent'
```

_And so on for other possible tools._

## Code Checks

Before submitting a pull request, please ensure your code passes all checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo check
```

## Commit Conventions

We use conventional commits. Please format your commit messages like so:

`type(scope): message`

### Types:

- `feat`: A new feature or tool
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to other unrelated tasks (build process, package manager, etc)

### Example:

`feat(channel): add create_forum_post tool`

## Questions?

If you have any questions or aren't sure how to implement something, feel free to open an issue to discuss it first. I'm happy to help!
