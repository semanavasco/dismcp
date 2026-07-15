# dismcp

Simple Discord MCP HTTP server built with:

- [`serenity`](https://crates.io/crates/serenity) for Discord API access
- [`rmcp`](https://crates.io/crates/rmcp) for MCP tool serving

> [!NOTE]
> The project is in early development. The goal is to provide a simple HTTP server that exposes Discord bot functionality via the MCP protocol. This allows for easy integration with other tools and services that support MCP (such as AI agents).
>
> The project is designed to be easily extensible, allowing developers to add more tools as needed.
>
> I plan to add more tools in the future, such as sending messages, managing channels, emojis, servers, and more. The end goal is to provide a comprehensive set of tools that cover most of the Discord bot functionality.

## Requirements

- Rust toolchain
- A Discord bot token

Environment variables:

- `DISCORD_TOKEN` (required)
- `MCP_BIND_ADDRESS` (optional, default: `127.0.0.1:3000`)

## Quick start

```bash
DISCORD_TOKEN=your_bot_token cargo run
```

If you want a custom bind:

```bash
MCP_BIND_ADDRESS=127.0.0.1:4000 DISCORD_TOKEN=your_bot_token cargo run
```

## Fast development test loop

Use these commands in a second terminal while the server is running to quickly test your tools.

Implemented tool categories:

- `guild`
- `user`

Current tools:

| Category | Tool                 | Description                                      |
| -------- | -------------------- | ------------------------------------------------ |
| guild    | `get_guilds`         | List guilds visible to the authenticated user.   |
| guild    | `get_guild`          | Get details for a guild by ID.                   |
| guild    | `get_guild_channels` | List channels in a guild.                        |
| guild    | `get_guild_members`  | List members in a guild with pagination support. |
| user     | `get_current_user`   | Get the authenticated user.                      |
| user     | `get_user`           | Get a user by ID.                                |

### Example usage

#### List tools

```bash
curl -sS http://127.0.0.1:3000 \
  -H 'content-type: application/json' \
  -H 'accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | jq
```

#### Call `get_guilds`

```bash
curl -sS http://127.0.0.1:3000 \
  -H 'content-type: application/json' \
  -H 'accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_guilds","arguments":{}}}' | jq '.result.structuredContent'
```

#### Call `get_current_user`

```bash
curl -sS http://127.0.0.1:3000 \
  -H 'content-type: application/json' \
  -H 'accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_current_user","arguments":{}}}' | jq '.result.structuredContent'
```

_And so on for other possible tools._

## Code checks

```bash
cargo fmt
cargo check
cargo clippy
```
