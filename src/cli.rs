use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Output format for linting results
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output with colors
    Human,
    /// JSON format for machine processing
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Human
    }
}

/// Command-line interface for the YL YAML linter
#[derive(Parser)]
#[command(
    name = "yl",
    about = "A YAML linter written in Rust",
    version = env!("CARGO_PKG_VERSION"),
    after_help = "For more information, see: https://github.com/scottidler/yl"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Files or directories to lint (when no subcommand is used)
    #[arg(help = "Files or directories to lint")]
    pub files: Vec<PathBuf>,

    /// Configuration file path
    #[arg(short, long, help = "Path to configuration file")]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "human", help = "Output format")]
    pub format: OutputFormat,

    /// Show only errors (no warnings)
    #[arg(long, help = "Show only errors, suppress warnings")]
    pub errors_only: bool,

    /// Disable specific rules
    #[arg(long, help = "Disable specific rules (comma-separated)")]
    pub disable: Vec<String>,

    /// Enable specific rules
    #[arg(long, help = "Enable specific rules (comma-separated)")]
    pub enable: Vec<String>,

    /// Set rule parameters (format: rule.param=value)
    #[arg(long, help = "Set rule parameters (format: rule.param=value)")]
    pub set: Vec<String>,

    /// List all available rules and exit
    #[arg(long, help = "List all available rules and exit")]
    pub list_rules: bool,

    /// Show configuration and exit
    #[arg(long, help = "Show effective configuration and exit")]
    pub show_config: bool,

    /// Enable verbose output
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
}

/// Available subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Start the Language Server Protocol (LSP) server
    Lsp,
    /// Fix auto-fixable problems in files
    Fix {
        /// Files or directories to fix
        files: Vec<PathBuf>,
        /// Show what would be fixed without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Migrate from yamllint configuration and directives
    Migrate {
        #[command(subcommand)]
        migrate_command: MigrateCommands,
    },
    /// Plugin management
    Plugin {
        #[command(subcommand)]
        plugin_command: PluginCommands,
    },
}

/// Migration subcommands
#[derive(Subcommand)]
pub enum MigrateCommands {
    /// Convert yamllint configuration to yl format
    Config {
        /// Path to yamllint configuration file
        input: PathBuf,
        /// Output path for yl configuration
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Convert yamllint directives in YAML files
    Directives {
        /// Files or directories to convert
        files: Vec<PathBuf>,
    },
    /// Migrate entire project from yamllint to yl
    Project {
        /// Project directory path
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

/// Plugin subcommands
#[derive(Subcommand)]
pub enum PluginCommands {
    /// List loaded plugins
    List,
    /// Load plugins from directory
    Load {
        /// Directory containing plugin libraries
        directory: PathBuf,
    },
}

impl Cli {
    /// Parse disable rules from comma-separated string
    pub fn get_disabled_rules(&self) -> Vec<String> {
        self.disable
            .iter()
            .flat_map(|s| s.split(','))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Parse enable rules from comma-separated string
    pub fn get_enabled_rules(&self) -> Vec<String> {
        self.enable
            .iter()
            .flat_map(|s| s.split(','))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Parse rule parameter settings
    pub fn get_rule_settings(&self) -> Vec<(String, String, String)> {
        self.set
            .iter()
            .filter_map(|s| {
                let parts: Vec<&str> = s.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key_parts: Vec<&str> = parts[0].splitn(2, '.').collect();
                    if key_parts.len() == 2 {
                        Some((
                            key_parts[0].trim().to_string(), // rule
                            key_parts[1].trim().to_string(), // param
                            parts[1].trim().to_string(),     // value
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get files to process, defaulting to current directory if none specified
    pub fn get_files(&self) -> Vec<PathBuf> {
        if self.files.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            self.files.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_default() {
        assert!(matches!(OutputFormat::default(), OutputFormat::Human));
    }

    #[test]
    fn test_get_disabled_rules() {
        let cli = Cli {
            disable: vec!["rule1,rule2".to_string(), "rule3".to_string()],
            ..Default::default()
        };

        let disabled = cli.get_disabled_rules();
        assert_eq!(disabled, vec!["rule1", "rule2", "rule3"]);
    }

    #[test]
    fn test_get_enabled_rules() {
        let cli = Cli {
            enable: vec!["rule1,rule2".to_string(), "rule3".to_string()],
            ..Default::default()
        };

        let enabled = cli.get_enabled_rules();
        assert_eq!(enabled, vec!["rule1", "rule2", "rule3"]);
    }

    #[test]
    fn test_get_rule_settings() {
        let cli = Cli {
            set: vec![
                "line-length.max=120".to_string(),
                "indentation.spaces=4".to_string(),
                "invalid-setting".to_string(), // Should be ignored
            ],
            ..Default::default()
        };

        let settings = cli.get_rule_settings();
        assert_eq!(settings.len(), 2);
        assert_eq!(
            settings[0],
            ("line-length".to_string(), "max".to_string(), "120".to_string())
        );
        assert_eq!(
            settings[1],
            ("indentation".to_string(), "spaces".to_string(), "4".to_string())
        );
    }

    #[test]
    fn test_get_files_default() {
        let cli = Cli {
            files: vec![],
            ..Default::default()
        };

        let files = cli.get_files();
        assert_eq!(files, vec![PathBuf::from(".")]);
    }

    #[test]
    fn test_get_files_specified() {
        let cli = Cli {
            files: vec![PathBuf::from("file1.yaml"), PathBuf::from("file2.yaml")],
            ..Default::default()
        };

        let files = cli.get_files();
        assert_eq!(files, vec![PathBuf::from("file1.yaml"), PathBuf::from("file2.yaml")]);
    }
}

// Provide a default implementation for testing
impl Default for Cli {
    fn default() -> Self {
        Self {
            command: None,
            files: Vec::new(),
            config: None,
            format: OutputFormat::default(),
            errors_only: false,
            disable: Vec::new(),
            enable: Vec::new(),
            set: Vec::new(),
            list_rules: false,
            show_config: false,
            verbose: false,
        }
    }
}
