use crate::models::Torrent;
use crate::utils::{extract_chinese_title, format_size, get_category_name, get_torrent_url, has_chinese_subtitle};
use console::Term;

/// 输出格式化器
pub struct Printer {
    term: Term,
}

impl Printer {
    /// 创建新的打印器
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    /// 打印欢迎界面
    pub fn print_welcome(&self) {
        println!("\n🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟");
        println!("🌸                                                    🌸");
        println!("🌸          欢迎使用 M-Team 种子查询工具 v2.0         🌸");
        println!("🌸                    by 幽浮喵 ฅ'ω'ฅ                  🌸");
        println!("🌸                                                    🌸");
        println!("🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟🌟\n");
    }

    /// 打印提示信息
    pub fn print_tips(&self) {
        println!("💡 提示:");
        println!("  - 直接输入关键词开始搜索");
        println!("  - 输入 'menu' 打开主菜单");
        println!("  - 输入 'browse' 按分类浏览");
        println!("  - 输入 'settings' 修改默认设置");
        println!("  - 输入 'exit' 或 'quit' 退出程序\n");
    }

    /// 打印种子列表表格
    pub fn print_table(&self, torrents: &[Torrent], base_url: &str) {
        if torrents.is_empty() {
            println!("📭 没有找到相关种子喵～\n");
            return;
        }

        println!("\n{}", "=".repeat(150));
        println!(
            "{:<6} {:<45} {:<10} {:<7} {:<7} {:<7} {:<50}",
            "序号", "标题", "大小", "做种", "下载", "分类", "种子页面链接"
        );
        println!("{}", "=".repeat(150));

        for (idx, torrent) in torrents.iter().enumerate() {
            let title = extract_chinese_title(torrent);
            let title = if title.len() > 42 {
                format!("{}...", &title[..42])
            } else {
                title
            };

            let size = format_size(&torrent.size);
            let seeders = torrent.status.seeders.to_string();
            let leechers = torrent.status.leechers.to_string();
            let category = get_category_name(&torrent.category);
            let torrent_url = get_torrent_url(base_url, &torrent.id);

            // 中文字幕标记
            let prefix = if has_chinese_subtitle(&torrent.name, &torrent.labels_new) {
                "💖"
            } else {
                "  "
            };

            println!(
                "{:<6} {:<45} {:<10} {:<7} {:<7} {:<7} {:<50}",
                format!("[{}]", idx + 1),
                format!("{} {}", prefix, title),
                size,
                seeders,
                leechers,
                category,
                torrent_url
            );
        }

        println!("{}\n", "=".repeat(150));
    }

    /// 打印种子详情
    pub fn print_detail(&self, torrent: &Torrent) {
        println!("\n{}", "═".repeat(60));
        println!("📦 种子详情");
        println!("{}", "═".repeat(60));

        // 优先显示中文标题
        if let Some(ref descr) = torrent.small_descr {
            println!("标题: {}", descr);
        }
        println!("文件名: {}", torrent.name);
        println!("大小: {}", format_size(&torrent.size));
        println!("分类: {}", get_category_name(&torrent.category));
        println!("ID: {}", torrent.id);
        println!("做种数: {}", torrent.status.seeders);
        println!("下载数: {}", torrent.status.leechers);
        println!("完成数: {}", torrent.status.completed);

        if let Some(ref imdb) = torrent.imdb {
            println!("IMDb: {}", imdb);
        }
        if let Some(ref douban) = torrent.douban {
            println!("豆瓣: {}", douban);
        }

        println!("{}\n", "═".repeat(60));
    }

    /// 打印错误信息
    pub fn print_error(&self, msg: &str) {
        eprintln!("❌ {}", msg);
    }

    /// 打印成功信息
    pub fn print_success(&self, msg: &str) {
        println!("✅ {}", msg);
    }

    /// 打印信息
    pub fn print_info(&self, msg: &str) {
        println!("ℹ️  {}", msg);
    }

    /// 打印主菜单
    pub fn print_main_menu(&self) {
        println!("\n{}", "=".repeat(60));
        println!("📋 主菜单");
        println!("{}", "=".repeat(60));
        println!("  1. 🔍 搜索种子");
        println!("  2. 📂 按分类浏览");
        println!("  3. ⚙️  设置");
        println!("  0. 🚪 退出");
        println!("{}", "=".repeat(60));
    }

    /// 打印分类菜单
    pub fn print_category_menu(&self) {
        println!("\n{}", "=".repeat(60));
        println!("📂 分类浏览");
        println!("{}", "=".repeat(60));

        use crate::models::CATEGORY_MAP;
        for (idx, (id, name, _)) in CATEGORY_MAP.iter().enumerate() {
            println!("  {}. {} ({})", idx + 1, name, id);
        }

        println!("  0. 🚪 返回");
        println!("{}", "=".repeat(60));
    }

    /// 打印设置菜单
    pub fn print_settings_menu(&self, default_category: Option<u32>, default_sort: &str, default_limit: usize) {
        println!("\n{}", "=".repeat(60));
        println!("⚙️  默认设置");
        println!("{}", "=".repeat(60));
        println!("  1. 分类筛选: {}", default_category.map_or("全部".to_string(), |c| c.to_string()));
        println!("  2. 排序方式: {}", default_sort);
        println!("  3. 结果数量: {}", default_limit);
        println!("  0. 返回主菜单");
        println!("{}", "=".repeat(60));
    }

    /// 打印帮助信息
    pub fn print_help(&self) {
        println!("\n{}", "=".repeat(60));
        println!("📖 命令帮助");
        println!("{}", "=".repeat(60));
        println!("快速搜索:");
        println!("  直接输入关键词即可搜索");
        println!("\n快捷命令:");
        println!("  help      - 显示此帮助信息");
        println!("  menu      - 打开主菜单");
        println!("  browse    - 按分类浏览");
        println!("  settings  - 修改默认设置");
        println!("  exit/quit - 退出程序");
        println!("{}", "=".repeat(60));
    }
}

impl Default for Printer {
    fn default() -> Self {
        Self::new()
    }
}
