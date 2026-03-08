use crate::client::MTeamClient;
use crate::models::{CATEGORY_MAP, SearchMode, SortDirection, SortField};
use crate::printer::Printer;
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::path::PathBuf;

/// 交互式菜单
pub struct InteractiveMenu {
    client: MTeamClient,
    printer: Printer,
    default_category: Option<u32>,
    default_sort: SortField,
    default_limit: usize,
    current_torrents: Vec<crate::models::Torrent>,
}

impl InteractiveMenu {
    /// 创建新的交互式菜单
    pub fn new(client: MTeamClient) -> Self {
        Self {
            client,
            printer: Printer::new(),
            default_category: None,
            default_sort: SortField::Seeders,
            default_limit: 20,
            current_torrents: Vec::new(),
        }
    }

    /// 运行快速搜索模式（启动默认模式）
    pub async fn run_quick_search_mode(&mut self) -> Result<()> {
        self.printer.print_welcome();
        self.printer.print_tips();

        loop {
            let keyword: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("\n🔎 搜索")
                .allow_empty(true)
                .interact_text()?;

            let keyword = keyword.trim();

            // 处理快捷命令
            match keyword.to_lowercase().as_str() {
                "" => continue,
                "exit" | "quit" | "0" | "q" => {
                    println!("\n👋 感谢使用,再见喵～ ฅ'ω'ฅ\n");
                    break;
                }
                "menu" => {
                    self.run_main_menu_mode().await?;
                    continue;
                }
                "browse" => {
                    self.run_browse_mode().await?;
                    continue;
                }
                "settings" => {
                    self.run_settings_mode().await?;
                    continue;
                }
                "help" => {
                    self.printer.print_help();
                    continue;
                }
                _ => {
                    // 执行搜索
                    self.perform_search(keyword).await?;
                }
            }
        }

        Ok(())
    }

    /// 运行主菜单模式
    pub async fn run_main_menu_mode(&mut self) -> Result<()> {
        loop {
            self.printer.print_main_menu();

            let items = vec![
                "🔍 搜索种子",
                "📂 按分类浏览",
                "⚙️  设置",
                "🚪 退出",
            ];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .items(&items)
                .default(0)
                .interact()?;

            match selection {
                0 => self.run_search_mode().await?,
                1 => self.run_browse_mode().await?,
                2 => self.run_settings_mode().await?,
                3 => {
                    println!("\n👋 感谢使用,再见喵～ ฅ'ω'ฅ\n");
                    break;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    /// 搜索模式
    async fn run_search_mode(&mut self) -> Result<()> {
        println!("\n{}", "🔍".repeat(30));
        println!("进入搜索模式");
        println!("{}", "🔍".repeat(30));

        let keyword: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("\n请输入搜索关键词")
            .allow_empty(true)
            .interact_text()?;

        if keyword.is_empty() || keyword == "0" {
            println!("取消搜索");
            return Ok(());
        }

        self.perform_search(&keyword).await
    }

    /// 分类浏览模式
    async fn run_browse_mode(&mut self) -> Result<()> {
        println!("\n{}", "📂".repeat(30));
        println!("进入分类浏览模式");
        println!("{}", "📂".repeat(30));

        self.printer.print_category_menu();

        // 构建分类选项
        let mut items = vec!["🚪 返回".to_string()];
        for (id, name, _) in CATEGORY_MAP {
            items.push(format!("{} ({})", name, id));
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()?;

        if selection == 0 {
            return Ok(());
        }

        // 获取选中的分类
        let category_id = CATEGORY_MAP[selection - 1].0;
        let category_id: u32 = category_id.parse().context("分类 ID 无效")?;

        let category_name = CATEGORY_MAP[selection - 1].1;
        println!("\n正在浏览分类: {}", category_name);

        // 执行搜索
        let mode = if category_id == 429 {
            SearchMode::Adult
        } else {
            SearchMode::Normal
        };

        let torrents = self
            .client
            .search(
                "",
                Some(category_id),
                mode,
                self.default_sort,
                SortDirection::Desc,
                self.default_limit,
            )
            .await
            .context("搜索失败")?;

        self.current_torrents = torrents;
        self.printer.print_table(&self.current_torrents, self.client.base_url());

        if !self.current_torrents.is_empty() {
            // 分类浏览模式：简化版种子选择，只允许下载和查看详情
            self.handle_browse_selection().await;
        }

        Ok(())
    }

    /// 分类浏览模式下的种子选择（简化版，避免递归）
    async fn handle_browse_selection(&mut self) {
        loop {
            println!("\n操作选项:");
            println!("  1. 📥 下载种子文件");
            println!("  2. 🔗 获取下载链接");
            println!("  3. 📄 查看种子详情");
            println!("  0. 🚪 返回");

            let choice: String = match Input::with_theme(&ColorfulTheme::default())
                .with_prompt("请选择操作")
                .interact_text()
            {
                Ok(c) => c,
                Err(_) => return,
            };

            match choice.trim() {
                "0" => return,
                "1" => { let _ = self.download_torrent().await; }
                "2" => { let _ = self.get_download_link().await; }
                "3" => { let _ = self.view_detail().await; }
                _ => println!("无效的选择，请重试"),
            }
        }
    }

    /// 设置模式
    async fn run_settings_mode(&mut self) -> Result<()> {
        loop {
            self.printer.print_settings_menu(
                self.default_category,
                self.get_sort_field_name(),
                self.default_limit,
            );

            let items = vec![
                format!("分类筛选: {}", self.get_category_display()),
                format!("排序方式: {}", self.get_sort_field_name()),
                format!("结果数量: {}", self.default_limit),
                "返回主菜单".to_string(),
            ];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .items(&items)
                .default(0)
                .interact()?;

            match selection {
                0 => {
                    // 修改分类
                    self.change_category().await?;
                }
                1 => {
                    // 修改排序
                    self.change_sort().await?;
                }
                2 => {
                    // 修改数量
                    self.change_limit().await?;
                }
                3 => break,
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    /// 执行搜索
    async fn perform_search(&mut self, keyword: &str) -> Result<()> {
        loop {
            println!("\n正在搜索: {}...", keyword);
            println!(
                "使用默认设置: 分类={}, 排序={}, 数量={}",
                self.default_category.map_or("全部".to_string(), |c| c.to_string()),
                self.get_sort_field_name(),
                self.default_limit
            );

            // 判断是否使用成人模式
            let mode = if self.default_category == None || self.default_category == Some(429) {
                SearchMode::Adult
            } else {
                SearchMode::Normal
            };

            let torrents = self
                .client
                .search(
                    keyword,
                    self.default_category,
                    mode,
                    self.default_sort,
                    SortDirection::Desc,
                    self.default_limit,
                )
                .await
                .context("搜索失败")?;

            self.current_torrents = torrents;
            self.printer.print_table(&self.current_torrents, self.client.base_url());

            if !self.current_torrents.is_empty() {
                // 处理种子选择操作
                match self.handle_torrent_selection().await? {
                    Some(true) => {
                        // 需要重新搜索，提示输入新关键词
                        let new_keyword: String = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("\n请输入新的搜索关键词")
                            .allow_empty(true)
                            .interact_text()?;

                        if new_keyword.is_empty() || new_keyword == "0" {
                            break;
                        }

                        // 继续循环，使用新关键词搜索
                        continue;
                    }
                    Some(false) => break,
                    None => continue,
                }
            }

            break;
        }

        Ok(())
    }

    /// 处理种子选择和操作
    /// 返回 Some(true) 表示需要重新搜索，Some(false) 表示正常返回，None 表示继续循环
    async fn handle_torrent_selection(&mut self) -> Result<Option<bool>> {
        loop {
            println!("\n操作选项:");
            println!("  1. 📥 下载种子文件");
            println!("  2. 🔗 获取下载链接");
            println!("  3. 📄 查看种子详情");
            println!("  4. 🔍 重新搜索");
            println!("  5. 📂 按分类浏览");
            println!("  6. ⚙️  默认设置");
            println!("  0. 🚪 返回");

            let choice: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("请选择操作")
                .interact_text()?;

            match choice.trim() {
                "0" => return Ok(Some(false)),
                "1" => self.download_torrent().await?,
                "2" => self.get_download_link().await?,
                "3" => self.view_detail().await?,
                "4" => return Ok(Some(true)), // 返回 true 表示需要重新搜索
                "5" => {
                    // 不递归调用 run_browse_mode，而是直接返回让外层处理
                    return Ok(Some(false));
                }
                "6" => {
                    // 不递归调用 run_settings_mode，直接返回
                    return Ok(Some(false));
                }
                _ => println!("无效的选择，请重试"),
            }
        }
    }

    /// 下载种子文件
    async fn download_torrent(&mut self) -> Result<()> {
        if self.current_torrents.is_empty() {
            println!("没有可下载的种子");
            return Ok(());
        }

        let index: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("请输入种子序号")
            .interact_text()?;

        let index: usize = index.trim().parse().context("序号无效")?;

        if index == 0 || index > self.current_torrents.len() {
            println!("❌ 序号超出范围");
            return Ok(());
        }

        let torrent = &self.current_torrents[index - 1];

        println!("\n正在下载种子文件: {}", torrent.id);

        let save_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("保存路径 (直接回车使用当前目录)")
            .default(".".to_string())
            .interact_text()?;

        let save_path = PathBuf::from(save_path);

        match self
            .client
            .download_torrent(&torrent.id, &save_path)
            .await
        {
            Ok(path) => {
                self.printer.print_success(&format!("下载完成: {}", path.display()));
            }
            Err(e) => {
                self.printer.print_error(&format!("下载失败: {}", e));
            }
        }

        Ok(())
    }

    /// 获取下载链接
    async fn get_download_link(&mut self) -> Result<()> {
        if self.current_torrents.is_empty() {
            println!("没有可获取链接的种子");
            return Ok(());
        }

        let index: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("请输入种子序号")
            .interact_text()?;

        let index: usize = index.trim().parse().context("序号无效")?;

        if index == 0 || index > self.current_torrents.len() {
            println!("❌ 序号超出范围");
            return Ok(());
        }

        let torrent = &self.current_torrents[index - 1];

        match self.client.get_download_url(&torrent.id).await {
            Ok(url) => {
                println!("\n✅ 下载链接: {}", url);
            }
            Err(e) => {
                self.printer.print_error(&format!("获取链接失败: {}", e));
            }
        }

        Ok(())
    }

    /// 查看详情
    async fn view_detail(&mut self) -> Result<()> {
        if self.current_torrents.is_empty() {
            println!("没有可查看的种子");
            return Ok(());
        }

        let index: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("请输入种子序号")
            .interact_text()?;

        let index: usize = index.trim().parse().context("序号无效")?;

        if index == 0 || index > self.current_torrents.len() {
            println!("❌ 序号超出范围");
            return Ok(());
        }

        let torrent = &self.current_torrents[index - 1];
        self.printer.print_detail(torrent);

        Ok(())
    }

    /// 修改分类
    async fn change_category(&mut self) -> Result<()> {
        let mut items = vec!["全部 (None)".to_string()];
        for (_, name, id) in CATEGORY_MAP {
            items.push(format!("{} ({})", name, id));
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()?;

        if selection == 0 {
            self.default_category = None;
            println!("✅ 已设置为: 全部");
        } else {
            let id = CATEGORY_MAP[selection - 1].0;
            self.default_category = Some(id.parse().context("分类 ID 无效")?);
            println!("✅ 已设置为: {}", items[selection]);
        }

        Ok(())
    }

    /// 修改排序
    async fn change_sort(&mut self) -> Result<()> {
        let items = vec!["做种数", "文件大小", "发布时间"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()?;

        self.default_sort = match selection {
            0 => SortField::Seeders,
            1 => SortField::Size,
            2 => SortField::Time,
            _ => unreachable!(),
        };

        println!("✅ 已设置为: {}", items[selection]);

        Ok(())
    }

    /// 修改数量
    async fn change_limit(&mut self) -> Result<()> {
        let limit: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("请输入结果数量 (1-100)")
            .default(self.default_limit.to_string())
            .interact_text()?;

        let limit: usize = limit.trim().parse().context("数量无效")?;

        if limit < 1 || limit > 100 {
            println!("❌ 数量必须在 1-100 之间");
            return Ok(());
        }

        self.default_limit = limit;
        println!("✅ 已设置为: {}", limit);

        Ok(())
    }

    /// 获取排序字段名称
    fn get_sort_field_name(&self) -> &str {
        match self.default_sort {
            SortField::Seeders => "seeders",
            SortField::Size => "size",
            SortField::Time => "time",
        }
    }

    /// 获取分类显示名称
    fn get_category_display(&self) -> String {
        match self.default_category {
            None => "全部".to_string(),
            Some(cat) => {
                for (id, name, _) in CATEGORY_MAP {
                    if *id == cat.to_string() {
                        return format!("{} ({})", name, id);
                    }
                }
                cat.to_string()
            }
        }
    }
}
