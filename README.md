# dismcp

Simple Discord MCP server built with:

- [`serenity`](https://crates.io/crates/serenity) for Discord API access
- [`rmcp`](https://crates.io/crates/rmcp) for MCP tool serving

> [!NOTE]
> The project is in early development. The goal is to provide a simple MCP server that exposes Discord bot functionality. This allows for easy integration with other tools and services that support MCP (such as AI agents).
>
> The project is designed to be easily extensible, allowing developers to add more tools as needed. The end goal is to provide a comprehensive set of tools that cover most of the Discord bot functionality.

## Requirements

- Rust toolchain (cargo)
- A Discord bot token (obtained from the [Discord Developer Portal](https://discord.com/developers/applications))

Environment variables:

- `DISCORD_TOKEN` (required)
- `MCP_TRANSPORT` (optional, default: `stdio`. Accepts `stdio` or `http`)
- `MCP_BIND_ADDRESS` (optional, default: `127.0.0.1:3000`. Used only if transport is `http`)
- `MCP_ENABLED_TOOLS` (optional, default: `all`. Accepts `all` or a comma-separated list of categories, e.g., `channel,guild,message`)
- `MCP_OMIT_NULLS` (optional, default: `false`. When set to `true`, all `null` fields are stripped from tool responses to reduce token consumption for AI agents)

## Quick start

By default, the server runs in **stdio** mode, which is meant to be executed directly by AI clients:

```bash
DISCORD_TOKEN=your_bot_token cargo run
```

If you want to run an **HTTP** server for remote access:

```bash
MCP_TRANSPORT=http DISCORD_TOKEN=your_bot_token cargo run
```

If you want a custom bind for HTTP:

```bash
MCP_TRANSPORT=http MCP_BIND_ADDRESS=127.0.0.1:4000/mcp DISCORD_TOKEN=your_bot_token cargo run
```

Or if you install via `cargo install dismcp` or `cargo install --path .`, you can use the binary directly:

```bash
DISCORD_TOKEN=your_bot_token dismcp
```

## How to wire it to your AI agent

Most AI clients expect MCP servers to be executed as background subprocesses communicating over standard input/output (`stdio`).

### 1. Install or locate the binary

The easiest way is to install it globally:

```bash
cargo install dismcp
```

### 2. Wire it to your AI agent

<details>
<summary><strong>Claude Code</strong></summary>

Run the following command to add the MCP server to your Claude Code project:

```json
claude mcp add discord dismcp -e DISCORD_TOKEN=your_bot_token
```

</details>

<details>
<summary><strong>Cursor</strong></summary>

Add to `.cursor/mcp.json` in your project (or the global settings):

```json
{
  "mcpServers": {
    "discord": {
      "command": "dismcp",
      "env": {
        "DISCORD_TOKEN": "your_bot_token"
      }
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
  "mcpServers": {
    "discord": {
      "command": "dismcp",
      "env": {
        "DISCORD_TOKEN": "your_bot_token"
      }
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
      "command": "dismcp",
      "env": {
        "DISCORD_TOKEN": "your_bot_token"
      }
    }
  }
}
```

</details>

<details>
<summary><strong>Using HTTP mode instead?</strong></summary>

If your client only supports HTTP, or you are running `dismcp` on a different machine, run the server with `MCP_TRANSPORT=http` and configure your client to point to the URL (e.g. `http://127.0.0.1:3000/`).
</details>

<details>
<summary><strong>Other clients</strong></summary>

Refer to their documentation for how to add a custom MCP server.

</details>

### 3. That's it

Once wired, the agent sees all Discord tools (channels, messages, roles, emojis, etc.) and can call them directly. For example, you could ask the agent:

> _"List my Discord servers and find the #general channel in my test server"_

and it would call `get_guilds` & `search_guild_channels` autonomously.

## Implemented tool categories:

- [application](#application)
- [channel](#channel)
- [emoji](#emoji)
- [guild](#guild)
- [member](#member)
- [message](#message)
- [role](#role)
- [user](#user)
- [webhook](#webhook)

### application

| Tool                      | Description                                     |
| ------------------------- | ----------------------------------------------- |
| `get_current_application` | Get info about the current Discord application. |
| `edit_bot_profile`        | Edit the bot username, avatar, or banner.       |

### channel

| Tool                                          | Description                                        |
| --------------------------------------------- | -------------------------------------------------- |
| `search_guild_channels`                       | Search channels in a guild by name/type.           |
| `get_channel`                                 | Get details about one channel by ID.               |
| `get_dm_channels`                             | List DM channels for the authenticated user.       |
| `create_dm_channel`                           | Create/get a DM channel with a target user.        |
| `get_guild_channels`                          | List channels in a guild.                          |
| `create_guild_channel`                        | Create a guild channel (text, voice, forum, etc).  |
| `edit_channel`                                | Edit a channel, including typed forum fields.      |
| `delete_channel`                              | Delete a guild or DM channel.                      |
| `create_forum_post`                           | Create a post in a forum channel.                  |
| `edit_forum_post_tags`                        | Replace applied tags on a forum post/thread.       |
| `get_guild_active_threads`                    | List active threads/posts in a guild.              |
| `get_channel_archived_public_threads`         | List archived public threads/posts in a channel.   |
| `get_channel_archived_private_threads`        | List archived private threads in a channel.        |
| `get_channel_joined_archived_private_threads` | List joined archived private threads in a channel. |
| `get_channel_thread_members`                  | List members currently in a thread channel.        |
| `join_thread_channel`                         | Join a thread channel.                             |
| `leave_thread_channel`                        | Leave a thread channel.                            |
| `get_channel_invites`                         | List invites for a channel.                        |
| `create_channel_invite`                       | Create an invite for a channel.                    |
| `delete_invite`                               | Delete an invite by code.                          |

### emoji

| Tool                       | Description                                      |
| -------------------------- | ------------------------------------------------ |
| `get_guild_emojis`         | List guild emojis.                               |
| `get_guild_emoji`          | Get one guild emoji by ID.                       |
| `create_guild_emoji`       | Create a guild emoji from a data URI image.      |
| `edit_guild_emoji`         | Edit a guild emoji name/roles.                   |
| `delete_guild_emoji`       | Delete a guild emoji.                            |
| `get_application_emojis`   | List application emojis.                         |
| `get_application_emoji`    | Get one application emoji by ID.                 |
| `create_application_emoji` | Create an application emoji from data URI image. |
| `edit_application_emoji`   | Edit an application emoji name.                  |
| `delete_application_emoji` | Delete an application emoji.                     |

### guild

| Tool                        | Description                                      |
| --------------------------- | ------------------------------------------------ |
| `get_guilds`                | List guilds visible to the authenticated user.   |
| `get_guild`                 | Get details for a guild by ID.                   |
| `get_guild_members`         | List members in a guild with pagination support. |
| `get_guild_invites`         | List all active invites in a guild.              |
| `get_scheduled_events`      | List all scheduled events in a guild.            |
| `get_scheduled_event`       | Get details for a scheduled event by ID.         |
| `create_scheduled_event`    | Create a new scheduled event.                    |
| `edit_scheduled_event`      | Edit an existing scheduled event.                |
| `delete_scheduled_event`    | Delete a scheduled event.                        |
| `get_scheduled_event_users` | List users subscribed to a scheduled event.      |

### member

| Tool                   | Description                             |
| ---------------------- | --------------------------------------- |
| `get_member`           | Get one guild member by guild/user IDs. |
| `kick_member`          | Kick a member from a guild.             |
| `ban_member`           | Ban a user from a guild.                |
| `unban_member`         | Unban a user from a guild.              |
| `timeout_member`       | Set a member timeout until a timestamp. |
| `clear_member_timeout` | Remove a member timeout.                |

### message

| Tool                            | Description                                    |
| ------------------------------- | ---------------------------------------------- |
| `get_message`                   | Get one message by channel/message IDs.        |
| `get_messages`                  | List channel messages with pagination.         |
| `send_message`                  | Send a rich message to guild channels or DMs.  |
| `edit_message`                  | Edit a rich message in guild channels or DMs.  |
| `delete_message`                | Delete a message in guild channels or DMs.     |
| `get_pinned_messages`           | List pinned messages in a channel.             |
| `pin_message`                   | Pin a message in a channel.                    |
| `unpin_message`                 | Unpin a message in a channel.                  |
| `add_message_reaction`          | Add a reaction to a message.                   |
| `remove_own_message_reaction`   | Remove the bot's reaction from a message.      |
| `remove_user_message_reaction`  | Remove another user's reaction from a message. |
| `clear_message_reactions`       | Clear all reactions from a message.            |
| `clear_message_emoji_reactions` | Clear one emoji's reactions from a message.    |
| `get_message_reaction_users`    | List users who reacted with a specific emoji.  |

### role

| Tool                       | Description                        |
| -------------------------- | ---------------------------------- |
| `search_guild_roles`       | Search roles in a guild by name.   |
| `get_guild_roles`          | List roles in a guild.             |
| `get_guild_role`           | Get one role by ID.                |
| `create_guild_role`        | Create a role in a guild.          |
| `edit_guild_role`          | Edit an existing guild role.       |
| `delete_guild_role`        | Delete a guild role.               |
| `edit_guild_role_position` | Edit role ordering/position.       |
| `add_member_role`          | Assign a role to a guild member.   |
| `remove_member_role`       | Remove a role from a guild member. |

### user

| Tool                   | Description                                      |
| ---------------------- | ------------------------------------------------ |
| `search_guild_members` | Search members by username/global name/nickname. |
| `get_current_user`     | Get the authenticated user.                      |
| `get_user`             | Get a user by ID.                                |

### webhook

| Tool                   | Description                                  |
| ---------------------- | -------------------------------------------- |
| `get_channel_webhooks` | List all webhooks in a specific channel.     |
| `get_guild_webhooks`   | List all webhooks in a specific guild.       |
| `get_webhook`          | Get details for a specific webhook by ID.    |
| `create_webhook`       | Create a new webhook for a specific channel. |
| `edit_webhook`         | Edit an existing webhook by ID.              |
| `delete_webhook`       | Delete a webhook by ID.                      |

## Contributing

I welcome contributions! If you'd like to help improve `dismcp`, please see the [Contributing Guidelines](CONTRIBUTING.md) for details on how to get started, add new tools, and submit your pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
