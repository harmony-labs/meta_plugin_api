use std::any::Any;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Failed to load plugin: {0}")]
    LoadError(String),
    #[error("Command not found: {0}")]
    CommandNotFound(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpMode {
    /// Plugin help completely replaces system help
    Override,
    /// Plugin help is prepended before system help
    Prepend,
    /// Plugin does not customize help
    None,
}

pub trait Plugin: Any {
    fn name(&self) -> &'static str;
    fn commands(&self) -> Vec<&'static str>;
    fn execute(&self, command: &str, args: &[String]) -> anyhow::Result<()>;

    /// Provide custom help output.
    /// Return Some((HelpMode, help text)) to customize help,
    /// or None to fallback to system help.
    fn get_help_output(&self, _args: &[String]) -> Option<(HelpMode, String)> {
        None
    }
}

pub type PluginCreate = unsafe fn() -> *mut dyn Plugin;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};

    struct MockSuccessPlugin;
    impl Plugin for MockSuccessPlugin {
        fn name(&self) -> &'static str {
            "mock_success"
        }
        fn commands(&self) -> Vec<&'static str> {
            vec!["success_cmd"]
        }
        fn execute(&self, command: &str, _args: &[String]) -> Result<()> {
            if command == "success_cmd" {
                Ok(())
            } else {
                Err(anyhow!("Command not found"))
            }
        }
    }

    struct MockFailPlugin;
    impl Plugin for MockFailPlugin {
        fn name(&self) -> &'static str {
            "mock_fail"
        }
        fn commands(&self) -> Vec<&'static str> {
            vec!["fail_cmd"]
        }
        fn execute(&self, _command: &str, _args: &[String]) -> Result<()> {
            Err(anyhow!("Simulated plugin failure"))
        }
    }

    #[test]
    fn test_plugin_loading_success() {
        // Simulate successful plugin creation
        let plugin: Box<dyn Plugin> = Box::new(MockSuccessPlugin);
        assert_eq!(plugin.name(), "mock_success");
        assert!(plugin.commands().contains(&"success_cmd"));
    }

    #[test]
    fn test_plugin_loading_failure() {
        // Simulate plugin load failure error
        let err = PluginError::LoadError("dlopen failed".to_string());
        assert_eq!(format!("{}", err), "Failed to load plugin: dlopen failed");
    }

    #[test]
    fn test_plugin_execute_success() {
        let plugin = MockSuccessPlugin;
        let result = plugin.execute("success_cmd", &[]);
        assert!(result.is_ok());
    }
    
    pub use crate::Plugin;
    pub use crate::HelpMode;
    pub use crate::PluginError;

    #[test]
    fn test_plugin_execute_command_not_found() {
        let plugin = MockSuccessPlugin;
        let result = plugin.execute("unknown_cmd", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_execute_failure() {
        let plugin = MockFailPlugin;
        let result = plugin.execute("fail_cmd", &[]);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Simulated plugin failure"));
    }
}