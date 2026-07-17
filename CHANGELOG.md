# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-16

### Added

- Initial release of `dismcp`, a lightweight Discord MCP server with around 80 tools!
- Stateless HTTP Transport: Compatible with most AI agents via standard HTTP POST, removing the need for complex SSE connections.
- Application Tools: `get_current_application`, `edit_bot_profile`.
- Channel Tools: Search, fetch, create, edit, and delete text, voice, forum, and DM channels. Includes thread management and channel invites.
- Emoji Tools: Full CRUD operations for both guild and application emojis.
- Guild Tools: Fetch guilds, members, and invites. Includes full CRUD operations for scheduled events and event subscriptions.
- Member Tools: Fetch, kick, ban, unban, and timeout management.
- Message Tools: Send, edit, delete, pin/unpin messages, and manage message reactions.
- Role Tools: Search, fetch, create, edit, delete, and manage role hierarchy and assignments.
- User Tools: Search members and fetch user profiles.
- Webhook Tools: Full CRUD operations for managing webhooks in channels and guilds.

[0.1.0]: https://github.com/semanavasco/dismcp/releases/tag/v0.1.0
