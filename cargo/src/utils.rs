use crate::models::{Torrent, SortField, SortDirection};

/// 文件大小格式化
pub fn format_size(size: &str) -> String {
    match size.parse::<u64>() {
        Ok(bytes) if bytes > 0 => {
            const GB: u64 = 1024 * 1024 * 1024;
            const MB: u64 = 1024 * 1024;
            const KB: u64 = 1024;

            if bytes >= GB {
                format!("{:.2} GB", bytes as f64 / GB as f64)
            } else if bytes >= MB {
                format!("{:.2} MB", bytes as f64 / MB as f64)
            } else if bytes >= KB {
                format!("{:.2} KB", bytes as f64 / KB as f64)
            } else {
                format!("{} B", bytes)
            }
        }
        _ => "未知".to_string(),
    }
}

/// 判断是否包含中文字幕
pub fn has_chinese_subtitle(name: &str, labels: &[String]) -> bool {
    // 检查标签
    for label in labels {
        if label.contains("中") || label.contains("国配") {
            return true;
        }
    }

    // 检查文件名
    let chinese_keywords = [
        "中字", "中文", "国配", "国语", "繁体", "简体",
        "CHS", "CHT", "TC", "SC", "C-Movie", "CD1",
    ];

    let name_upper = name.to_uppercase();
    for keyword in &chinese_keywords {
        if name_upper.contains(keyword) || name.contains(keyword) {
            return true;
        }
    }

    false
}

/// 生成种子页面 URL
pub fn get_torrent_url(base_url: &str, torrent_id: &str) -> String {
    format!("{}/detail/{}", base_url, torrent_id)
}

/// 排序种子列表
pub fn sort_torrents(
    torrents: &mut Vec<Torrent>,
    field: SortField,
    direction: SortDirection,
) {
    match field {
        SortField::Seeders => {
            torrents.sort_by(|a, b| {
                let a_seeders = a.status.seeders;
                let b_seeders = b.status.seeders;
                let a_has_chinese = has_chinese_subtitle(&a.name, &a.labels_new);
                let b_has_chinese = has_chinese_subtitle(&b.name, &b.labels_new);

                // 中文字幕优先
                match (a_has_chinese, b_has_chinese) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }

                // 然后按做种数排序
                let ord = if direction == SortDirection::Desc {
                    b_seeders.cmp(&a_seeders)
                } else {
                    a_seeders.cmp(&b_seeders)
                };
                ord
            });
        }
        SortField::Size => {
            torrents.sort_by(|a, b| {
                let a_size = a.size.parse::<u64>().unwrap_or(0);
                let b_size = b.size.parse::<u64>().unwrap_or(0);

                if direction == SortDirection::Desc {
                    b_size.cmp(&a_size)
                } else {
                    a_size.cmp(&b_size)
                }
            });
        }
        SortField::Time => {
            torrents.sort_by(|a, b| {
                // 使用 ID 作为时间排序依据（ID 越大越新）
                let a_id = a.id.parse::<u64>().unwrap_or(0);
                let b_id = b.id.parse::<u64>().unwrap_or(0);

                if direction == SortDirection::Desc {
                    b_id.cmp(&a_id)
                } else {
                    a_id.cmp(&b_id)
                }
            });
        }
    }
}

/// 提取中文标题
pub fn extract_chinese_title(torrent: &Torrent) -> String {
    if let Some(ref descr) = torrent.small_descr {
        // 提取到第一个 | 之前的内容
        descr.split('|')
            .next()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| torrent.name.clone())
    } else {
        torrent.name.clone()
    }
}

/// 获取分类名称
pub fn get_category_name(category_id: &str) -> &'static str {
    use crate::models::CATEGORY_MAP;
    for (id, name, _) in CATEGORY_MAP {
        if *id == category_id {
            return name;
        }
    }
    "未知"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TorrentStatus;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size("1073741824"), "1.00 GB");
        assert_eq!(format_size("1048576"), "1.00 MB");
        assert_eq!(format_size("1024"), "1.00 KB");
        assert_eq!(format_size("512"), "512 B");
        assert_eq!(format_size("invalid"), "未知");
    }

    #[test]
    fn test_has_chinese_subtitle() {
        assert!(has_chinese_subtitle("Test.中字", &[]));
        assert!(has_chinese_subtitle("Test", &["中字".to_string()]));
        assert!(!has_chinese_subtitle("Test", &[]));
    }

    #[test]
    fn test_extract_chinese_title() {
        let torrent = Torrent {
            id: "1".to_string(),
            name: "Dune Part Two".to_string(),
            small_descr: Some("沙丘2 | 4K HDR | ...".to_string()),
            size: "0".to_string(),
            category: "419".to_string(),
            labels_new: vec![],
            status: TorrentStatus {
                id: "1".to_string(),
                seeders: 0,
                leechers: 0,
                completed: 0,
                status: "NORMAL".to_string(),
            },
            created_date: None,
            imdb: None,
            douban: None,
        };

        assert_eq!(extract_chinese_title(&torrent), "沙丘2");
    }
}
