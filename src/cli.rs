//! Application cli arguments handling.

const HELP: &str = concat!("\
dismcp - ", env!("CARGO_PKG_DESCRIPTION"), ".

USAGE:
    dismcp [OPTIONS]

OPTIONS:
    -h, --help       Print this help message and exit
    -V, --version    Print the version and exit

ENVIRONMENT VARIABLES:
    DISCORD_TOKEN       (required) Discord bot token
    MCP_TRANSPORT       (optional, default: stdio) Transport mode: 'stdio' or 'http'
    MCP_BIND_ADDRESS    (optional, default: 127.0.0.1:3000) HTTP bind address (only used if MCP_TRANSPORT=http)
    MCP_ENABLED_TOOLS   (optional, default: all) Comma-separated list of tool categories to enable
    MCP_OMIT_NULLS      (optional, default: false) Strip null fields from responses to reduce token usage

EXAMPLES:
    DISCORD_TOKEN=your_token dismcp
    MCP_TRANSPORT=http MCP_BIND_ADDRESS=127.0.0.1:3000/mcp DISCORD_TOKEN=your_token dismcp
    MCP_OMIT_NULLS=true DISCORD_TOKEN=your_token dismcp

For more information, visit: https://github.com/semanavasco/dismcp");

/// Handles command-line arguments for the `dismcp` application.
pub(crate) fn handle_args() {
    let mut args = std::env::args().skip(1);

    if let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("dismcp {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {other}");
                eprintln!("Run 'dismcp --help' for usage information.");
                std::process::exit(1);
            }
        }
    }
}
