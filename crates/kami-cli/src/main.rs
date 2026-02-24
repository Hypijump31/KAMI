//! KAMI CLI - Orchestrateur WASM/MCP.

use clap::{Parser, Subcommand};
use tracing_subscriber::fmt::format::FmtSpan;

mod commands;
mod input;
mod output;
pub(crate) mod shared;

/// KAMI - Secure WASM tool orchestrator for AI agents.
#[derive(Debug, Parser)]
#[command(name = "kami", version, about)]
struct Cli {
    /// Configuration file path.
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Verbosity level (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Log output format: plain (default) or json (for log aggregation).
    #[arg(long, global = true, default_value = "plain", value_parser = ["plain", "json"])]
    log_format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Build a WASM tool from source.
    Build(commands::build::BuildArgs),
    /// Scaffold a new tool project.
    Init(commands::init::InitArgs),
    /// Validate a tool.toml manifest without installing.
    Validate(commands::validate::ValidateArgs),
    /// Install a WASM tool into the registry.
    Install(commands::install::InstallArgs),
    /// Uninstall a tool from the registry.
    Uninstall(commands::uninstall::UninstallArgs),
    /// Update a tool (or all) from their source directories.
    Update(commands::update::UpdateArgs),
    /// Search for tools in the remote registry.
    Search(commands::search::SearchArgs),
    /// Generate a registry entry to publish your tool.
    Publish(commands::publish::PublishArgs),
    /// Generate an Ed25519 signing keypair.
    Keygen(commands::keygen::KeygenArgs),
    /// Sign a WASM plugin with your Ed25519 key.
    Sign(commands::sign::SignArgs),
    /// Pin a tool to a specific version (prevents update).
    Pin(commands::pin::PinArgs),
    /// List installed tools.
    List(commands::list::ListArgs),
    /// Inspect a tool's manifest.
    Inspect(commands::inspect::InspectArgs),
    /// Run a WASM component file directly (dev mode).
    Run(commands::run::RunArgs),
    /// Execute a registered tool by ID.
    Exec(commands::exec::ExecArgs),
    /// Start MCP server over stdio or HTTP.
    Serve(commands::serve::ServeArgs),
    /// Show runtime status and tool registry statistics.
    Status(commands::status::StatusArgs),
    /// Verify the integrity of an installed tool's WASM file.
    Verify(commands::verify::VerifyArgs),
    /// Developer experience commands (watch, etc.).
    Dev(commands::dev::DevArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing.
    let filter = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    match cli.log_format.as_str() {
        "json" => tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .with_target(true)
            .with_span_events(FmtSpan::CLOSE)
            .init(),
        _ => tracing_subscriber::fmt().with_env_filter(filter).init(),
    };

    tracing::debug!("KAMI starting with config: {:?}", cli.config);

    match &cli.command {
        Commands::Build(args) => commands::build::execute(args),
        Commands::Init(args) => commands::init::execute(args),
        Commands::Validate(args) => commands::validate::execute(args),
        Commands::Install(args) => commands::install::execute(args).await,
        Commands::Uninstall(args) => commands::uninstall::execute(args).await,
        Commands::Update(args) => commands::update::execute(args).await,
        Commands::Search(args) => commands::search::execute(args).await,
        Commands::Publish(args) => commands::publish::execute(args),
        Commands::Keygen(args) => commands::keygen::execute(args),
        Commands::Sign(args) => commands::sign::execute(args),
        Commands::Pin(args) => commands::pin::execute(args).await,
        Commands::List(args) => commands::list::execute(args).await,
        Commands::Inspect(args) => commands::inspect::execute(args).await,
        Commands::Run(args) => commands::run::execute(args).await,
        Commands::Exec(args) => commands::exec::execute(args).await,
        Commands::Serve(args) => commands::serve::execute(args).await,
        Commands::Status(args) => commands::status::execute(args).await,
        Commands::Verify(args) => commands::verify::execute(args).await,
        Commands::Dev(args) => commands::dev::execute(args).await,
    }
}
