use anyhow::Result;
use colored::Colorize;
use inquire::{Confirm, CustomType, Select, Text};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::config::{ClientConfig, Config, Settings, save_config};

pub fn run() -> Result<()> {
    println!();
    println!("{}", "gtt — Git Time Tracker".bold().cyan());
    println!("{}", "Setup interactivo de configuración".dimmed());
    println!();

    let mut config = Config {
        client: HashMap::new(),
        settings: Settings::default(),
    };

    // Session gap
    let gap: u32 = CustomType::new("¿Minutos de inactividad para iniciar nueva sesión?")
        .with_default(120u32)
        .with_help_message("Default 120 min (2 horas). Ajusta según tu estilo de trabajo.")
        .prompt()?;
    config.settings.session_gap_minutes = gap;

    // First commit time
    let first: u32 = CustomType::new("¿Minutos base por primer commit de sesión?")
        .with_default(30u32)
        .with_help_message("Tiempo que se asume trabajado antes del primer commit. Default 30 min.")
        .prompt()?;
    config.settings.first_commit_minutes = first;

    // Add clients
    loop {
        println!();
        let client_name = Text::new("Nombre del cliente (vacío para terminar):")
            .prompt()?;
        if client_name.trim().is_empty() {
            break;
        }

        let mut repos: Vec<PathBuf> = Vec::new();
        loop {
            let repo_path = Text::new("  Ruta al repositorio (vacío para terminar):")
                .with_help_message("Ruta absoluta, ej: /home/user/mi-proyecto")
                .prompt()?;
            if repo_path.trim().is_empty() {
                break;
            }
            let path = PathBuf::from(repo_path.trim());
            if !path.exists() {
                println!("{}", "  Advertencia: la ruta no existe (se agregará de todas formas)".yellow());
            }
            repos.push(path);
        }

        let rate: f64 = CustomType::new("  Tasa horaria (0 para no configurar):")
            .with_default(0.0f64)
            .prompt()?;

        let currency = if rate > 0.0 {
            let options = vec!["USD", "EUR", "GBP", "ARS", "MXN", "Otra"];
            let sel = Select::new("  Moneda:", options).prompt()?;
            if sel == "Otra" {
                Text::new("  Código de moneda (ej: CLP):").prompt()?
            } else {
                sel.to_string()
            }
        } else {
            "USD".to_string()
        };

        config.client.insert(
            client_name.trim().to_string(),
            ClientConfig {
                repos,
                hourly_rate: rate,
                currency,
            },
        );

        let add_more = Confirm::new("¿Agregar otro cliente?")
            .with_default(false)
            .prompt()?;
        if !add_more {
            break;
        }
    }

    if config.client.is_empty() {
        println!("{}", "\nNo se configuró ningún cliente. Puedes editar el config manualmente.".yellow());
    }

    save_config(&config)?;

    let config_path = crate::config::config_path()?;
    println!();
    println!("{}", "Configuración guardada exitosamente.".green().bold());
    println!("  {}", config_path.display().to_string().dimmed());
    println!();
    println!("Próximos pasos:");
    println!("  {}  Ver horas de hoy y esta semana", "gtt status".cyan());
    println!("  {}  Ver reporte del mes anterior", "gtt report --last-month".cyan());
    println!();

    Ok(())
}
