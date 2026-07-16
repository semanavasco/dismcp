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

**Types:**

- `feat`: A new feature or tool
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to other unrelated tasks (build process, package manager, etc)

**Example:**
`feat(channel): add create_forum_post tool`

## Questions?

If you have any questions or aren't sure how to implement something, feel free to open an issue to discuss it first. I'm happy to help!
