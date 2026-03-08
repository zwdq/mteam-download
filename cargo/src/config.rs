use crate::models::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// 配置管理器
pub struct ConfigManager;

impl ConfigManager {
    /// 从配置文件加载配置
    pub fn load() -> Result<Config> {
        // 1. 尝试从当前目录加载
        let config_path = PathBuf::from("config.yaml");
        if config_path.exists() {
            return Self::load_from_file(&config_path);
        }

        // 2. 尝试从上级目录加载（兼容 cargo 子目录情况）
        let parent_config = PathBuf::from("../config.yaml");
        if parent_config.exists() {
            return Self::load_from_file(&parent_config);
        }

        // 3. 尝试从用户主目录加载
        if let Some(home_dir) = dirs::home_dir() {
            let home_config = home_dir.join(".config").join("mteam-query").join("config.yaml");
            if home_config.exists() {
                return Self::load_from_file(&home_config);
            }
        }

        Err(anyhow::anyhow!(
            "找不到配置文件 config.yaml\n\
            请确保配置文件存在于以下位置之一：\n\
            - 当前目录: ./config.yaml\n\
            - 上级目录: ../config.yaml\n\
            - 用户配置: ~/.config/mteam-query/config.yaml"
        ))
    }

    /// 从指定文件加载配置
    fn load_from_file(path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("无法读取配置文件: {}", path.display()))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("配置文件格式错误: {}", path.display()))?;

        // 打印配置信息
        eprintln!("🔍 [DEBUG] 从 {} 加载配置", path.display());
        eprintln!("🔍 [DEBUG] API Key: {}***", &config.api_key[..8]);
        eprintln!("🔍 [DEBUG] 代理配置: {:?}", config.proxies);

        // 验证 API Key
        if config.api_key.is_empty() || config.api_key == "your_mteam_api_key_here" {
            return Err(anyhow::anyhow!(
                "API Key 未配置或无效\n\
                请在 config.yaml 中填入有效的 M-Team API Key"
            ));
        }

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(config: &Config, path: &Path) -> Result<()> {
        let yaml = serde_yaml::to_string(config)
            .context("配置序列化失败")?;

        fs::write(path, yaml)
            .with_context(|| format!("无法写入配置文件: {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_invalid_config() {
        // 测试不存在的配置文件
        let result = ConfigManager::load();
        assert!(result.is_err());
    }
}
