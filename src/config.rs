use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
};

use crate::cli::ConfigCommands;

// ── Provider configuration ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_max_tokens() -> u32 {
    4096
}

impl ProviderConfig {
    pub fn openai_default() -> Self {
        Self {
            api_key: None,
            model: "gpt-4o".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            max_tokens: 4096,
        }
    }

    pub fn anthropic_default() -> Self {
        Self {
            api_key: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            max_tokens: 4096,
        }
    }

    pub fn ollama_default() -> Self {
        Self {
            api_key: None,
            model: "llama3.2".to_string(),
            base_url: "http://localhost:11434".to_string(),
            max_tokens: 4096,
        }
    }
}

// ── Top-level config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_provider: String,
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub system_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert("openai".to_string(), ProviderConfig::openai_default());
        providers.insert("anthropic".to_string(), ProviderConfig::anthropic_default());
        providers.insert("ollama".to_string(), ProviderConfig::ollama_default());

        Self {
            default_provider: "openai".to_string(),
            providers,
            stream: true,
            system_prompt: None,
        }
    }
}

impl Config {
    /// Path to the configuration file
    pub fn path() -> Result<PathBuf> {
        let dir = if let Ok(custom) = std::env::var("TGPT_CONFIG_DIR") {
            PathBuf::from(custom)
        } else if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("tgpt")
        } else if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".tgpt")
        } else {
            PathBuf::from(".tgpt")
        };
        fs::create_dir_all(&dir)?;
        Ok(dir.join("config.json"))
    }

    /// Load config from disk, creating defaults if absent
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            let cfg = Config::default();
            cfg.save()?;
            return Ok(cfg);
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let cfg: Config = serde_json::from_str(&raw)
            .with_context(|| format!("Invalid JSON in config file {}", path.display()))?;
        Ok(cfg)
    }

    /// Persist config to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Return the ProviderConfig for the given provider name
    pub fn provider(&self, name: &str) -> Result<&ProviderConfig> {
        self.providers
            .get(name)
            .with_context(|| format!("Unknown provider '{}'. Known providers: {}", name, self.providers.keys().cloned().collect::<Vec<_>>().join(", ")))
    }

    /// Return mutable ProviderConfig
    pub fn provider_mut(&mut self, name: &str) -> &mut ProviderConfig {
        self.providers
            .entry(name.to_string())
            .or_insert_with(|| ProviderConfig::openai_default())
    }

    /// Resolve which provider to use: CLI override → config default
    #[allow(dead_code)]
    pub fn resolve_provider<'a>(&'a self, override_: Option<&'a str>) -> &'a str {
        override_.unwrap_or(&self.default_provider)
    }
}

// ── Config command handler ────────────────────────────────────────────────────

pub fn handle_config_command(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::SetKey { provider, key } => {
            let mut cfg = Config::load()?;
            cfg.provider_mut(&provider).api_key = Some(key.clone());
            cfg.save()?;
            println!(
                "{} API key set for {}",
                "✓".green().bold(),
                provider.cyan().bold()
            );
            if key.len() > 8 {
                println!("  Key: {}…{}", &key[..4], &key[key.len() - 4..]);
            }
        }

        ConfigCommands::SetModel { provider, model } => {
            let mut cfg = Config::load()?;
            cfg.provider_mut(&provider).model = model.clone();
            cfg.save()?;
            println!(
                "{} Model set to {} for {}",
                "✓".green().bold(),
                model.cyan(),
                provider.cyan().bold()
            );
        }

        ConfigCommands::SetProvider { provider } => {
            let mut cfg = Config::load()?;
            cfg.default_provider = provider.clone();
            cfg.save()?;
            println!(
                "{} Default provider set to {}",
                "✓".green().bold(),
                provider.cyan().bold()
            );
        }

        ConfigCommands::SetUrl { provider, url } => {
            let mut cfg = Config::load()?;
            cfg.provider_mut(&provider).base_url = url.clone();
            cfg.save()?;
            println!(
                "{} Base URL set to {} for {}",
                "✓".green().bold(),
                url.cyan(),
                provider.cyan().bold()
            );
        }

        ConfigCommands::Show => {
            let cfg = Config::load()?;
            let path = Config::path()?;
            println!("\n{}", "tgpt Configuration".bold().underline());
            println!("  Config file : {}", path.display().to_string().dimmed());
            println!(
                "  Default     : {}",
                cfg.default_provider.cyan().bold()
            );
            println!("  Streaming   : {}", cfg.stream.to_string().cyan());
            println!();
            println!("{}", "Providers:".bold());
            for (name, p) in &cfg.providers {
                let marker = if name == &cfg.default_provider { " ← default" } else { "" };
                let key_status = match &p.api_key {
                    Some(k) if !k.is_empty() => "✓ set".green().to_string(),
                    _ => "✗ not set".red().to_string(),
                };
                println!("  {}{}", name.cyan().bold(), marker.dimmed());
                println!("    model   : {}", p.model);
                println!("    api_key : {}", key_status);
                println!("    url     : {}", p.base_url.dimmed());
            }
            println!();
        }

        ConfigCommands::Edit => {
            let path = Config::path()?;
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| {
                    if cfg!(windows) { "notepad".to_string() } else { "nano".to_string() }
                });
            std::process::Command::new(&editor)
                .arg(&path)
                .status()
                .with_context(|| format!("Failed to open editor '{}'", editor))?;
        }

        ConfigCommands::Reset => {
            let path = Config::path()?;
            if path.exists() {
                fs::remove_file(&path)?;
            }
            Config::default().save()?;
            println!("{} Configuration reset to defaults", "✓".green().bold());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.default_provider, "openai");
        assert!(cfg.stream);
        assert!(cfg.providers.contains_key("openai"));
        assert!(cfg.providers.contains_key("anthropic"));
        assert!(cfg.providers.contains_key("ollama"));
    }

    #[test]
    fn test_provider_lookup() {
        let cfg = Config::default();
        let openai = cfg.provider("openai").unwrap();
        assert_eq!(openai.model, "gpt-4o");
        assert_eq!(openai.base_url, "https://api.openai.com/v1");

        let ollama = cfg.provider("ollama").unwrap();
        assert_eq!(ollama.model, "llama3.2");
        assert_eq!(ollama.base_url, "http://localhost:11434");

        assert!(cfg.provider("invalid_provider").is_err());
    }

    #[test]
    fn test_resolve_provider() {
        let cfg = Config::default();
        assert_eq!(cfg.resolve_provider(Some("anthropic")), "anthropic");
        assert_eq!(cfg.resolve_provider(None), "openai");
    }
}
