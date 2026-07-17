# dismcp

Simple Discord MCP HTTP server built with:

- [`serenity`](https://crates.io/crates/serenity) for Discord API access
- [`rmcp`](https://crates.io/crates/rmcp) for MCP tool serving

> [!NOTE]
> The project is in early development. The goal is to provide a simple HTTP server that exposes Discord bot functionality via the MCP protocol. This allows for easy integration with other tools and services that support MCP (such as AI agents).
>
> The project is designed to be easily extensible, allowing developers to add more tools as needed. The end goal is to provide a comprehensive set of tools that cover most of the Discord bot functionality.

## Requirements

- Rust toolchain (cargo)
- A Discord bot token

Environment variables:

- `DISCORD_TOKEN` (required)
- `MCP_BIND_ADDRESS` (optional, default: `127.0.0.1:3000`)
- `MCP_ENABLED_TOOLS` (optional, default: `all`. Accepts `all` or a comma-separated list of categories, e.g., `channel,guild,message`)

## Quick start

```bash
DISCORD_TOKEN=your_bot_token cargo run
```

If you want a custom bind:

```bash
MCP_BIND_ADDRESS=127.0.0.1:4000/mcp DISCORD_TOKEN=your_bot_token cargo run
```

Or if you install via `cargo install dismcp` or `cargo install --path .`:

```bash
DISCORD_TOKEN=your_bot_token dismcp
```

## Implemented tool categories:

- `application`
- `channel`
- `emoji`
- `guild`
- `message`
- `member`
- `role`
- `user`
- `webhook`

<details>
<summary><h3>Current tools:</h3></summary>

| Category    | Tool                                          | Description                                        |
| ----------- | --------------------------------------------- | -------------------------------------------------- |
| application | `get_current_application`                     | Get info about the current Discord application.    |
| application | `edit_bot_profile`                            | Edit the bot username, avatar, or banner.          |
| channel     | `search_guild_channels`                       | Search channels in a guild by name/type.           |
| channel     | `get_channel`                                 | Get details about one channel by ID.               |
| channel     | `get_dm_channels`                             | List DM channels for the authenticated user.       |
| channel     | `create_dm_channel`                           | Create/get a DM channel with a target user.        |
| channel     | `get_guild_channels`                          | List channels in a guild.                          |
| channel     | `create_guild_channel`                        | Create a guild channel (text, voice, forum, etc).  |
| channel     | `edit_channel`                                | Edit a channel, including typed forum fields.      |
| channel     | `delete_channel`                              | Delete a guild or DM channel.                      |
| channel     | `create_forum_post`                           | Create a post in a forum channel.                  |
| channel     | `edit_forum_post_tags`                        | Replace applied tags on a forum post/thread.       |
| channel     | `get_guild_active_threads`                    | List active threads/posts in a guild.              |
| channel     | `get_channel_archived_public_threads`         | List archived public threads/posts in a channel.   |
| channel     | `get_channel_archived_private_threads`        | List archived private threads in a channel.        |
| channel     | `get_channel_joined_archived_private_threads` | List joined archived private threads in a channel. |
| channel     | `get_channel_thread_members`                  | List members currently in a thread channel.        |
| channel     | `join_thread_channel`                         | Join a thread channel.                             |
| channel     | `leave_thread_channel`                        | Leave a thread channel.                            |
| channel     | `get_channel_invites`                         | List invites for a channel.                        |
| channel     | `create_channel_invite`                       | Create an invite for a channel.                    |
| channel     | `delete_invite`                               | Delete an invite by code.                          |
| emoji       | `get_guild_emojis`                            | List guild emojis.                                 |
| emoji       | `get_guild_emoji`                             | Get one guild emoji by ID.                         |
| emoji       | `create_guild_emoji`                          | Create a guild emoji from a data URI image.        |
| emoji       | `edit_guild_emoji`                            | Edit a guild emoji name/roles.                     |
| emoji       | `delete_guild_emoji`                          | Delete a guild emoji.                              |
| emoji       | `get_application_emojis`                      | List application emojis.                           |
| emoji       | `get_application_emoji`                       | Get one application emoji by ID.                   |
| emoji       | `create_application_emoji`                    | Create an application emoji from data URI image.   |
| emoji       | `edit_application_emoji`                      | Edit an application emoji name.                    |
| emoji       | `delete_application_emoji`                    | Delete an application emoji.                       |
| guild       | `get_guilds`                                  | List guilds visible to the authenticated user.     |
| guild       | `get_guild`                                   | Get details for a guild by ID.                     |
| guild       | `get_guild_members`                           | List members in a guild with pagination support.   |
| guild       | `get_guild_invites`                           | List all active invites in a guild.                |
| guild       | `get_scheduled_events`                        | List all scheduled events in a guild.              |
| guild       | `get_scheduled_event`                         | Get details for a scheduled event by ID.           |
| guild       | `create_scheduled_event`                      | Create a new scheduled event.                      |
| guild       | `edit_scheduled_event`                        | Edit an existing scheduled event.                  |
| guild       | `delete_scheduled_event`                      | Delete a scheduled event.                          |
| guild       | `get_scheduled_event_users`                   | List users subscribed to a scheduled event.        |
| member      | `get_member`                                  | Get one guild member by guild/user IDs.            |
| member      | `kick_member`                                 | Kick a member from a guild.                        |
| member      | `ban_member`                                  | Ban a user from a guild.                           |
| member      | `unban_member`                                | Unban a user from a guild.                         |
| member      | `timeout_member`                              | Set a member timeout until a timestamp.            |
| member      | `clear_member_timeout`                        | Remove a member timeout.                           |
| message     | `get_message`                                 | Get one message by channel/message IDs.            |
| message     | `get_messages`                                | List channel messages with pagination.             |
| message     | `send_message`                                | Send a rich message to guild channels or DMs.      |
| message     | `edit_message`                                | Edit a rich message in guild channels or DMs.      |
| message     | `delete_message`                              | Delete a message in guild channels or DMs.         |
| message     | `get_pinned_messages`                         | List pinned messages in a channel.                 |
| message     | `pin_message`                                 | Pin a message in a channel.                        |
| message     | `unpin_message`                               | Unpin a message in a channel.                      |
| message     | `add_message_reaction`                        | Add a reaction to a message.                       |
| message     | `remove_own_message_reaction`                 | Remove the bot's reaction from a message.          |
| message     | `remove_user_message_reaction`                | Remove another user's reaction from a message.     |
| message     | `clear_message_reactions`                     | Clear all reactions from a message.                |
| message     | `clear_message_emoji_reactions`               | Clear one emoji's reactions from a message.        |
| message     | `get_message_reaction_users`                  | List users who reacted with a specific emoji.      |
| role        | `search_guild_roles`                          | Search roles in a guild by name.                   |
| role        | `get_guild_roles`                             | List roles in a guild.                             |
| role        | `get_guild_role`                              | Get one role by ID.                                |
| role        | `create_guild_role`                           | Create a role in a guild.                          |
| role        | `edit_guild_role`                             | Edit an existing guild role.                       |
| role        | `delete_guild_role`                           | Delete a guild role.                               |
| role        | `edit_guild_role_position`                    | Edit role ordering/position.                       |
| role        | `add_member_role`                             | Assign a role to a guild member.                   |
| role        | `remove_member_role`                          | Remove a role from a guild member.                 |
| user        | `search_guild_members`                        | Search members by username/global name/nickname.   |
| user        | `get_current_user`                            | Get the authenticated user.                        |
| user        | `get_user`                                    | Get a user by ID.                                  |
| webhook     | `get_channel_webhooks`                        | List all webhooks in a specific channel.           |
| webhook     | `get_guild_webhooks`                          | List all webhooks in a specific guild.             |
| webhook     | `get_webhook`                                 | Get details for a specific webhook by ID.          |
| webhook     | `create_webhook`                              | Create a new webhook for a specific channel.       |
| webhook     | `edit_webhook`                                | Edit an existing webhook by ID.                    |
| webhook     | `delete_webhook`                              | Delete a webhook by ID.                            |

</details>

### Example usage

Use these commands in a second terminal while the server is running to quickly test your tools.

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

#### Call `get_guild` with a guild ID

```bash
curl -sS http://127.0.0.1:3000 \
  -H 'content-type: application/json' \
  -H 'accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_guild","arguments":{"guild_id":"1234567890"}}}' | jq '.result.structuredContent'
```

_And so on for other possible tools._

## How to wire it to your AI agent

### 1. Start dismcp

```bash
DISCORD_TOKEN=your_bot_token cargo run
# Server listens on http://127.0.0.1:3000/
```

### 2. Wire it to your AI agent

<details>
<summary><strong>Claude Desktop / Claude Code</strong></summary>

Add to `~/.claude/settings.json` (Claude Code) or the Claude Desktop config:

```json
{
  "mcpServers": {
    "discord": {
      "type": "url",
      "url": "http://127.0.0.1:3000/"
    }
  }
}
```

</details>

<details>
<summary><strong>Cursor</strong></summary>

Add to `.cursor/mcp.json` in your project (or the global settings):

```json
{
  "mcpServers": {
    "discord": {
      "url": "http://127.0.0.1:3000/"
    }
  }
}
```

</details>

<details>
<summary><strong>VS Code (Copilot / GitHub Copilot Chat)</strong></summary>

Add to `.vscode/mcp.json`:

```json
{
  "servers": {
    "discord": {
      "type": "http",
      "url": "http://127.0.0.1:3000/"
    }
  }
}
```

</details>

<details>
<summary><strong>Antigravity</strong></summary>

Add to `.agents/mcp.json` in your workspace (or better: to `~/gemini/antigravity/mcp_config.json` for better support):

```json
{
  "mcpServers": {
    "discord": {
      "url": "http://127.0.0.1:3000/"
    }
  }
}
```

</details>

<details>
<summary><strong>Other clients</strong></summary>

Refer to their documentation for how to add a custom MCP server.

</details>

### 3. That's it

Once wired, the agent sees all Discord tools (channels, messages, roles, emojis, etc.) and can call them directly. For example, you could ask the agent:

> _"List my Discord servers and find the #general channel in my test server"_

and it would call `get_guilds` & `search_guild_channels` autonomously.
