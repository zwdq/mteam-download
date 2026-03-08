use crate::models::{ApiResponse, SearchMode, SortDirection, SortField, Torrent};
use crate::utils::sort_torrents;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// M-Team API 客户端
pub struct MTeamClient {
    client: Client,
    api_key: String,
    api_base: String,
    base_url: String,
    timeout: u64,
    proxies: Option<reqwest::Proxy>,
}

impl MTeamClient {
    /// 创建新的 API 客户端
    pub fn new(
        api_key: String,
        api_base: String,
        base_url: String,
        timeout: u64,
        proxies: Option<(String, String)>,
    ) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(timeout));

        // 设置代理（分别设置 HTTP 和 HTTPS）
        if let Some((http_proxy, https_proxy)) = proxies {
            eprintln!("🔍 [DEBUG] 配置代理:");
            eprintln!("  HTTP:  {}", http_proxy);
            eprintln!("  HTTPS: {}", https_proxy);

            // reqwest 需要分别设置 HTTP 和 HTTPS 代理
            let http_proxy = reqwest::Proxy::http(&http_proxy)
                .context("HTTP 代理配置无效")?;
            let https_proxy = reqwest::Proxy::https(&https_proxy)
                .context("HTTPS 代理配置无效")?;

            eprintln!("🔍 [DEBUG] 代理已配置");
            client_builder = client_builder.proxy(http_proxy).proxy(https_proxy);
        } else {
            eprintln!("🔍 [DEBUG] 未使用代理");
        }

        let client = client_builder.build()?;

        Ok(Self {
            client,
            api_key,
            api_base,
            base_url,
            timeout,
            proxies: None, // 已在 client builder 中设置
        })
    }

    /// 搜索种子
    pub async fn search(
        &self,
        keyword: &str,
        category: Option<u32>,
        mode: SearchMode,
        sort_field: SortField,
        sort_direction: SortDirection,
        limit: usize,
    ) -> Result<Vec<Torrent>> {
        let payload = if mode == SearchMode::Adult {
            // 成人模式 payload
            serde_json::json!({
                "mode": "adult",
                "keyword": keyword,
                "categories": [],
                "pageNumber": 1,
                "pageSize": limit
            })
        } else {
            // 普通模式 payload
            let mut payload = serde_json::json!({
                "keyword": keyword,
                "visible": 1
            });

            if let Some(cat) = category {
                payload["category"] = serde_json::json!(cat);
            }

            payload
        };

        let url = format!("{}/torrent/search", self.api_base);
        eprintln!("🔍 [DEBUG] 请求 URL: {}", url);
        eprintln!("🔍 [DEBUG] Payload: {}", serde_json::to_string_pretty(&payload).unwrap_or_default());

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .json(&payload)
            .send()
            .await
            .context("API 请求失败")?;

        let status = response.status();
        let response_text = response.text().await.context("读取响应失败")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!("API 返回错误状态码: {}, 响应: {}", status, response_text));
        }

        let api_response: ApiResponse<Torrent> = serde_json::from_str(&response_text)
            .with_context(|| format!("解析响应失败, 原始响应: {}", response_text))?;

        if api_response.code != "0" {
            return Err(anyhow::anyhow!("API 返回错误: code={}, message={}", api_response.code, api_response.message));
        }

        let mut torrents = api_response.data.data_items;

        // 排序
        sort_torrents(&mut torrents, sort_field, sort_direction);

        // 限制数量
        torrents.truncate(limit);

        Ok(torrents)
    }

    /// 获取种子详情
    pub async fn get_detail(&self, id: &str) -> Result<Torrent> {
        let response = self
            .client
            .get(format!("{}/torrent/detail/{}", self.api_base, id))
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("API 请求失败")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("API 返回错误: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct DetailResponse {
            code: String,
            message: String,
            data: Option<Torrent>,
        }

        let resp: DetailResponse = response.json().await.context("解析响应失败")?;

        if resp.code != "0" {
            return Err(anyhow::anyhow!("API 错误: {}", resp.message));
        }

        resp.data.ok_or_else(|| anyhow::anyhow!("种子不存在"))
    }

    /// 获取下载链接
    pub async fn get_download_url(&self, id: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct DownloadTokenResponse {
            code: String,
            message: String,
            data: Option<String>,
        }

        #[derive(Serialize)]
        struct DownloadRequest {
            id: String,
        }

        let response = self
            .client
            .post(format!("{}/torrent/genDlToken", self.api_base))
            .header("x-api-key", &self.api_key)
            .json(&DownloadRequest { id: id.to_string() })
            .send()
            .await
            .context("API 请求失败")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("API 返回错误: {}", response.status()));
        }

        let resp: DownloadTokenResponse = response.json().await.context("解析响应失败")?;

        if resp.code != "0" {
            return Err(anyhow::anyhow!("API 错误: {}", resp.message));
        }

        let token = resp.data.ok_or_else(|| anyhow::anyhow!("获取下载令牌失败"))?;

        Ok(format!("{}/torrent/download/{}", self.api_base, token))
    }

    /// 下载种子文件
    pub async fn download_torrent(&self, id: &str, save_path: &std::path::Path) -> Result<std::path::PathBuf> {
        let download_url = self.get_download_url(id).await?;

        let response = self
            .client
            .get(&download_url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("下载种子文件失败")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("下载失败: {}", response.status()));
        }

        let bytes = response.bytes().await.context("读取响应失败")?;

        // 验证种子文件
        if !bytes.starts_with(b"d") {
            return Err(anyhow::anyhow!("下载的文件不是有效的种子文件"));
        }

        // 确保目录存在
        if let Some(parent) = save_path.parent() {
            std::fs::create_dir_all(parent).context("创建目录失败")?;
        }

        // 保存文件
        let filename = format!("mteam_{}.torrent", id);
        let file_path = save_path.join(&filename);

        std::fs::write(&file_path, bytes).context("写入文件失败")?;

        Ok(file_path)
    }

    /// 获取基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MTeamClient::new(
            "test_key".to_string(),
            "https://api.m-team.cc/api".to_string(),
            "https://kp.m-team.cc".to_string(),
            15,
            None,
        );
        assert!(client.is_ok());
    }
}
