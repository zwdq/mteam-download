mod client;
mod config;
mod menu;
mod models;
mod printer;
mod utils;

use clap::{Parser, Subcommand};
use client::MTeamClient;
use config::ConfigManager;
use models::{SearchMode, SortDirection, SortField};
use printer::Printer;

/// M-Team 种子查询工具
#[derive(Parser, Debug)]
#[command(name = "mteam-query")]
#[command(version = "2.0.0")]
#[command(author = "幽浮喵")]
#[command(about = "M-Team 种子查询工具 - Rust 实现", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 搜索种子
    Search {
        /// 搜索关键词
        keyword: String,
        /// 分类 ID
        #[arg(short = 'c', long)]
        category: Option<u32>,
        /// 排序字段 (seeders, size, time)
        #[arg(short = 's', long, default_value = "seeders")]
        sort: String,
        /// 返回结果数量
        #[arg(short = 'l', long, default_value = "20")]
        limit: usize,
    },
    /// 查看种子详情
    Detail {
        /// 种子 ID
        id: String,
    },
    /// 获取下载链接
    Url {
        /// 种子 ID
        id: String,
    },
    /// 下载种子文件
    Download {
        /// 种子 ID
        id: String,
        /// 保存路径
        #[arg(short = 'p', long, default_value = ".")]
        path: String,
    },
    /// 交互式模式
    #[command(name = "-i")]
    Interactive,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 加载配置
    let config = match ConfigManager::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    };

    // 创建 API 客户端
    let proxies = config.proxies.as_ref().and_then(|p| {
        match (&p.http, &p.https) {
            (Some(http), Some(https)) => Some((http.clone(), https.clone())),
            (Some(http), None) => Some((http.clone(), http.clone())),
            (None, Some(https)) => Some((https.clone(), https.clone())),
            (None, None) => None,
        }
    });

    let client = match MTeamClient::new(
        config.api_key.clone(),
        "https://api.m-team.cc/api".to_string(),
        "https://kp.m-team.cc".to_string(),
        config.timeout,
        proxies,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 创建客户端失败: {}", e);
            std::process::exit(1);
        }
    };

    // 处理命令
    match args.command {
        Some(Commands::Search {
            keyword,
            category,
            sort,
            limit,
        }) => {
            let sort_field = match sort.as_str() {
                "size" => SortField::Size,
                "time" => SortField::Time,
                _ => SortField::Seeders,
            };

            let mode = if category == None || category == Some(429) {
                SearchMode::Adult
            } else {
                SearchMode::Normal
            };

            match client
                .search(&keyword, category, mode, sort_field, SortDirection::Desc, limit)
                .await
            {
                Ok(torrents) => {
                    let printer = Printer::new();
                    printer.print_table(&torrents, client.base_url());
                }
                Err(e) => {
                    eprintln!("❌ 搜索失败: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Detail { id }) => {
            match client.get_detail(&id).await {
                Ok(torrent) => {
                    let printer = Printer::new();
                    printer.print_detail(&torrent);
                }
                Err(e) => {
                    eprintln!("❌ 获取详情失败: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Url { id }) => {
            match client.get_download_url(&id).await {
                Ok(url) => {
                    println!("✅ 下载链接: {}", url);
                }
                Err(e) => {
                    eprintln!("❌ 获取链接失败: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Download { id, path }) => {
            use std::path::Path;
            match client.download_torrent(&id, Path::new(&path)).await {
                Ok(file_path) => {
                    println!("✅ 下载完成: {}", file_path.display());
                }
                Err(e) => {
                    eprintln!("❌ 下载失败: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Interactive) | None => {
            // 默认：交互式模式
            let mut menu = menu::InteractiveMenu::new(client);
            if let Err(e) = menu.run_quick_search_mode().await {
                eprintln!("❌ 运行失败: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
