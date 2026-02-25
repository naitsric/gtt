mod commands;
mod config;
mod errors;
mod git;
mod output;
mod session;

use clap::{Parser, Subcommand};
use anyhow::Result;

use commands::config_cmd::ConfigAction;
use commands::{export, init, report, status, verify};
use report::ReportArgs;
use verify::VerifyArgs;
use export::ExportArgs;

#[derive(Parser)]
#[command(
    name = "gtt",
    about = "Git Time Tracker — Estima horas trabajadas desde commits de git",
    version,
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup interactivo: crea la configuración de clientes y repos
    Init,

    /// Resumen rápido: horas de hoy y esta semana por cliente
    Status,

    /// Reporte de horas por cliente (tabla, CSV o JSON)
    Report {
        /// Nombre del cliente (todos si no se especifica)
        #[arg(long)]
        client: Option<String>,

        /// Reporte de la semana pasada
        #[arg(long)]
        last_week: bool,

        /// Reporte del mes pasado
        #[arg(long)]
        last_month: bool,

        /// Fecha de inicio (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Fecha de fin (YYYY-MM-DD)
        #[arg(long)]
        until: Option<String>,

        /// Formato de salida: table, csv, json
        #[arg(long, default_value = "table")]
        format: String,

        /// Archivo de salida (default: stdout para table, archivo auto para csv/json)
        #[arg(long)]
        output: Option<String>,
    },

    /// Lista las sesiones detectadas con timestamps para validar antes de facturar
    Verify {
        /// Nombre del cliente
        #[arg(long)]
        client: Option<String>,

        /// Sesiones de la semana pasada
        #[arg(long)]
        last_week: bool,

        /// Sesiones del mes pasado
        #[arg(long)]
        last_month: bool,

        /// Fecha de inicio (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Fecha de fin (YYYY-MM-DD)
        #[arg(long)]
        until: Option<String>,
    },

    /// Exporta el reporte a archivo (alias de report --format)
    Export {
        /// Nombre del cliente
        #[arg(long)]
        client: Option<String>,

        /// Semana pasada
        #[arg(long)]
        last_week: bool,

        /// Mes pasado
        #[arg(long)]
        last_month: bool,

        /// Fecha de inicio (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Fecha de fin (YYYY-MM-DD)
        #[arg(long)]
        until: Option<String>,

        /// Formato: csv, json
        #[arg(long, default_value = "csv")]
        format: String,

        /// Archivo de salida (auto-generado si no se especifica)
        #[arg(long)]
        output: Option<String>,
    },

    /// Gestiona la configuración de gtt
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init => init::run(),

        Commands::Status => status::run(),

        Commands::Report {
            client,
            last_week,
            last_month,
            since,
            until,
            format,
            output,
        } => report::run(ReportArgs {
            client,
            last_week,
            last_month,
            since,
            until,
            format,
            output,
            verify_mode: false,
        }),

        Commands::Verify {
            client,
            last_week,
            last_month,
            since,
            until,
        } => verify::run(VerifyArgs {
            client,
            last_week,
            last_month,
            since,
            until,
        }),

        Commands::Export {
            client,
            last_week,
            last_month,
            since,
            until,
            format,
            output,
        } => export::run(ExportArgs {
            client,
            last_week,
            last_month,
            since,
            until,
            format,
            output,
        }),

        Commands::Config { action } => commands::config_cmd::run(action),
    }
}
