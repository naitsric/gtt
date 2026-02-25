use anyhow::{bail, Result};
use colored::Colorize;
use crate::config::config_path;

#[derive(Debug, clap::Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Open config in $EDITOR
    Edit,
}

pub fn run(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => show(),
        ConfigAction::Edit => edit(),
    }
}

fn show() -> Result<()> {
    let path = config_path()?;
    if !path.exists() {
        println!("{}", "No hay configuración. Ejecuta `gtt init`.".yellow());
        return Ok(());
    }
    let content = std::fs::read_to_string(&path)?;
    println!("{}", format!("# {}", path.display()).dimmed());
    println!("{}", content);
    Ok(())
}

fn edit() -> Result<()> {
    let path = config_path()?;
    if !path.exists() {
        bail!("No hay configuración. Ejecuta `gtt init` primero.");
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string());

    let status = std::process::Command::new(&editor)
        .arg(&path)
        .status()?;

    if !status.success() {
        bail!("El editor '{}' terminó con error.", editor);
    }

    Ok(())
}
