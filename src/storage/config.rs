use serde::{Deserialize, Serialize};
use crate::error::{FlowError, Result};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub ai_mode: String,
    pub version: String,
    pub user: UserConfig,
    pub ai: AIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub enabled: bool,
    pub provider: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

impl Config {
    pub fn new(name: String, ai_mode: String) -> Self {
        let user_name = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        
        let ai_enabled = ai_mode != "local";
        
        Self {
            name,
            ai_mode: ai_mode.clone(),
            version: "0.1.0".to_string(),
            user: UserConfig {
                name: user_name,
                email: None,
            },
            ai: AIConfig {
                enabled: ai_enabled,
                provider: if ai_enabled { ai_mode } else { "none".to_string() },
                api_key: None,
                model: None,
            },
        }
    }
    
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(FlowError::FileNotFound(path.to_string_lossy().to_string()));
        }
        
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        
        Ok(config)
    }
    
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn set_user_email(&mut self, email: String) {
        self.user.email = Some(email);
    }
    
    pub fn set_ai_api_key(&mut self, api_key: String) {
        self.ai.api_key = Some(api_key);
    }
    
    pub fn set_ai_model(&mut self, model: String) {
        self.ai.model = Some(model);
    }
    
    pub fn is_ai_enabled(&self) -> bool {
        self.ai.enabled && self.ai.api_key.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_config_creation() {
        let config = Config::new("test-repo".to_string(), "openai".to_string());
        
        assert_eq!(config.name, "test-repo");
        assert_eq!(config.ai_mode, "openai");
        assert!(config.ai.enabled);
        assert_eq!(config.ai.provider, "openai");
    }
    
    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let original_config = Config::new("test-repo".to_string(), "local".to_string());
        original_config.save(&config_path).unwrap();
        
        let loaded_config = Config::load(&config_path).unwrap();
        
        assert_eq!(original_config.name, loaded_config.name);
        assert_eq!(original_config.ai_mode, loaded_config.ai_mode);
        assert_eq!(original_config.ai.enabled, loaded_config.ai.enabled);
    }
    
    #[test]
    fn test_local_mode_disables_ai() {
        let config = Config::new("test-repo".to_string(), "local".to_string());
        
        assert!(!config.ai.enabled);
        assert_eq!(config.ai.provider, "none");
        assert!(!config.is_ai_enabled());
    }
}