use serde::{Deserialize, Serialize};

/// 配置结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub api_key: String,
    #[serde(default)]
    pub proxies: Option<Proxies>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 {
    15
}

/// 代理配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Proxies {
    pub http: Option<String>,
    pub https: Option<String>,
}

/// Torrent 状态
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TorrentStatus {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub seeders: u32,
    #[serde(default)]
    pub leechers: u32,
    #[serde(default)]
    pub completed: u32,
    #[serde(default)]
    pub status: String,
}

/// Torrent 数据结构
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Torrent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "smallDescr", default)]
    pub small_descr: Option<String>,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub category: String,
    #[serde(rename = "labelsNew", default)]
    pub labels_new: Vec<String>,
    #[serde(default)]
    pub status: TorrentStatus,
    #[serde(default)]
    pub created_date: Option<String>,
    #[serde(default)]
    pub imdb: Option<String>,
    #[serde(default)]
    pub douban: Option<String>,
}

/// API 响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResponseData<T> {
    #[serde(default = "default_string")]
    pub page_number: String,
    #[serde(default = "default_string")]
    pub page_size: String,
    #[serde(default = "default_string")]
    pub total: String,
    #[serde(rename = "data", default = "default_vec")]
    pub data_items: Vec<T>,
}

impl<T> Default for ResponseData<T> {
    fn default() -> Self {
        Self {
            page_number: default_string(),
            page_size: default_string(),
            total: default_string(),
            data_items: default_vec(),
        }
    }
}

fn default_string() -> String {
    String::new()
}

fn default_vec<T>() -> Vec<T> {
    Vec::new()
}

/// API 通用响应
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub data: ResponseData<T>,
}

/// 搜索模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Normal,
    Adult,
}

/// 排序字段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Seeders,
    Size,
    Time,
}

/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// 分类映射
pub struct CategoryInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub category_type: &'static str,
}

/// 分类映射表
pub const CATEGORY_MAP: &[(&str, &str, &str)] = &[
    ("401", "SD电影", "movie"),
    ("419", "HD电影", "movie"),
    ("420", "原盘/UHD", "movie"),
    ("404", "纪录片", "movie"),
    ("402", "剧集", "tv"),
    ("438", "剧集包", "tv"),
    ("405", "动漫", "anime"),
    ("403", "综艺", "variety"),
    ("418", "体育", "sports"),
    ("429", "成人", "adult"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialize() {
        let yaml = r#"
api_key: "test_key"
timeout: 30
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.timeout, 30);
    }

    #[test]
    fn test_torrent_deserialize() {
        let json = r#"{
    "id": "123",
    "name": "Test Torrent",
    "size": "1073741824",
    "category": "419",
    "status": {
        "seeders": 10,
        "leechers": 5,
        "completed": 100
    }
}"#;
        let torrent: Torrent = serde_json::from_str(json).unwrap();
        assert_eq!(torrent.id, "123");
        assert_eq!(torrent.name, "Test Torrent");
        assert_eq!(torrent.status.seeders, 10);
    }
}
