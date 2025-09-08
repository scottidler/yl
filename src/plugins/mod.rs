use crate::rules::{Rule, RuleConfig};
use eyre::Result;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;

/// Trait that plugins must implement to provide rules
pub trait RulePlugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &'static str;

    /// Get the plugin version
    fn version(&self) -> &'static str;

    /// Get the plugin description
    fn description(&self) -> &'static str;
}

/// Plugin manager for loading and managing rule plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn RulePlugin>>,
    libraries: Vec<Library>, // Keep libraries loaded
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            libraries: Vec::new(),
        }
    }

    /// Load a plugin from a shared library
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        unsafe {
            let lib = Library::new(path)?;

            // Get the plugin creation function
            let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn RulePlugin> =
                lib.get(b"create_plugin")?;

            let plugin_ptr = create_plugin();
            let plugin = Box::from_raw(plugin_ptr);

            let plugin_name = plugin.name().to_string();

            // Store the plugin and keep the library loaded
            self.plugins.insert(plugin_name, plugin);
            self.libraries.push(lib);
        }

        Ok(())
    }

    /// Get all loaded plugins
    pub fn plugins(&self) -> Vec<&dyn RulePlugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }

    /// Load plugins from a directory
    pub fn load_plugins_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<usize> {
        let dir = dir.as_ref();
        let mut loaded_count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Look for shared library files
            if let Some(extension) = path.extension() {
                let is_lib = match extension.to_str() {
                    Some("so") => true,    // Linux
                    Some("dylib") => true, // macOS
                    Some("dll") => true,   // Windows
                    _ => false,
                };

                if is_lib {
                    match self.load_plugin(&path) {
                        Ok(()) => {
                            loaded_count += 1;
                            eprintln!("Loaded plugin: {}", path.display());
                        }
                        Err(e) => {
                            eprintln!("Failed to load plugin {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(loaded_count)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Example built-in plugin for demonstration
#[allow(dead_code)]
pub struct ExamplePlugin;

impl RulePlugin for ExamplePlugin {
    fn name(&self) -> &'static str {
        "example-plugin"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Example plugin demonstrating the plugin system"
    }
}

/// Example rule for the example plugin
#[allow(dead_code)]
pub struct ExampleRule;

impl Rule for ExampleRule {
    fn id(&self) -> &'static str {
        "example-rule"
    }

    fn description(&self) -> &'static str {
        "Example rule from plugin system"
    }

    fn check(
        &self,
        context: &crate::linter::LintContext,
        _config: &RuleConfig,
    ) -> Result<Vec<crate::linter::Problem>> {
        let mut problems = Vec::new();

        // Example: Check for lines containing "TODO"
        for (line_no, line) in context.content.lines().enumerate() {
            if line.contains("TODO") {
                problems.push(crate::linter::Problem::new(
                    line_no + 1,
                    line.find("TODO").unwrap() + 1,
                    crate::linter::Level::Info,
                    self.id(),
                    "Found TODO comment".to_string(),
                ));
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        RuleConfig::new(false, crate::linter::Level::Info)
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

/// Macro for creating plugin exports (for use in plugin development)
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn create_plugin() -> *mut dyn $crate::plugins::RulePlugin {
            let plugin = <$plugin_type>::new();
            Box::into_raw(Box::new(plugin))
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugins().len(), 0);
    }

    #[test]
    fn test_example_plugin() {
        let plugin = ExamplePlugin;
        assert_eq!(plugin.name(), "example-plugin");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(!plugin.description().is_empty());
    }

    #[test]
    fn test_example_rule() {
        use crate::linter::LintContext;
        use std::path::PathBuf;

        let rule = ExampleRule;
        let path = PathBuf::from("test.yaml");
        let content = "key: value\n# TODO: fix this\nother: data";
        let context = LintContext::new(&path, content);
        let config = rule.default_config();

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].rule, "example-rule");
        assert_eq!(problems[0].line, 2);
        assert!(problems[0].message.contains("TODO"));
    }
}
