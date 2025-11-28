//! 用户配置管理模块
//! 
//! 负责保存和加载用户偏好设置，包括：
//! - 保存的 WAQL 查询语句
//! - UI 主题选择
//! - 字体大小设置
//! - 自定义关键词

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 配置文件名
const CONFIG_FILE_NAME: &str = "user_data.json";

/// 默认字体大小
const DEFAULT_FONT_SIZE: f32 = 18.0;

/// 用户配置结构体
/// 
/// 存储应用程序的所有用户自定义设置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserConfig {
    /// 保存的 WAQL 语句列表
    pub saved_queries: Vec<String>,
    /// 选择的主题名称
    pub theme_name: String,
    /// 字体大小
    pub fontsize: f32,
    /// 自定义关键词列表
    pub custom_keywords: Vec<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            saved_queries: Vec::new(),
            theme_name: "GRUVBOX".to_string(),
            fontsize: DEFAULT_FONT_SIZE,
            custom_keywords: Vec::new(),
        }
    }
}

impl UserConfig {
    /// 从文件加载配置
    /// 
    /// 如果文件不存在或读取失败，返回默认配置
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<UserConfig>(&content) {
                return config;
            }
        }
        Self::default()
    }

    /// 保存配置到文件
    /// 
    /// # Errors
    /// 
    /// 如果序列化或写入文件失败，返回错误
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, json)?;
        Ok(())
    }

    /// 获取配置文件路径
    /// 
    /// 配置文件存储在可执行文件同目录下
    fn get_config_path() -> PathBuf {
        let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        path.pop(); // 移除可执行文件名
        path.push(CONFIG_FILE_NAME);
        path
    }

    /// 添加保存的查询语句
    /// 
    /// 如果查询已存在，不会重复添加
    pub fn add_saved_query(&mut self, query: String) -> bool {
        if !self.saved_queries.contains(&query) {
            self.saved_queries.push(query);
            true
        } else {
            false
        }
    }

    /// 删除保存的查询语句
    pub fn remove_saved_query(&mut self, index: usize) -> Option<String> {
        if index < self.saved_queries.len() {
            Some(self.saved_queries.remove(index))
        } else {
            None
        }
    }

    /// 添加自定义关键词
    /// 
    /// 如果关键词已存在，不会重复添加
    pub fn add_custom_keyword(&mut self, keyword: String) -> bool {
        if !keyword.is_empty() && !self.custom_keywords.contains(&keyword) {
            self.custom_keywords.push(keyword);
            true
        } else {
            false
        }
    }

    /// 删除自定义关键词
    pub fn remove_custom_keyword(&mut self, index: usize) -> Option<String> {
        if index < self.custom_keywords.len() {
            Some(self.custom_keywords.remove(index))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UserConfig::default();
        assert_eq!(config.theme_name, "GRUVBOX");
        assert_eq!(config.fontsize, DEFAULT_FONT_SIZE);
        assert!(config.saved_queries.is_empty());
        assert!(config.custom_keywords.is_empty());
    }

    #[test]
    fn test_add_saved_query() {
        let mut config = UserConfig::default();
        assert!(config.add_saved_query("test query".to_string()));
        assert_eq!(config.saved_queries.len(), 1);
        assert!(!config.add_saved_query("test query".to_string()));
        assert_eq!(config.saved_queries.len(), 1);
    }

    #[test]
    fn test_add_custom_keyword() {
        let mut config = UserConfig::default();
        assert!(config.add_custom_keyword("keyword1".to_string()));
        assert_eq!(config.custom_keywords.len(), 1);
        assert!(!config.add_custom_keyword("keyword1".to_string()));
        assert_eq!(config.custom_keywords.len(), 1);
        assert!(!config.add_custom_keyword("".to_string()));
    }
}
