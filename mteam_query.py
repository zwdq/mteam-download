#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
M-Team 种子查询工具
使用 token/API Key 查询 M-Team 站点的种子资源
遵循 KISS、DRY、SOLID 原则设计
Author: 幽浮喵 (Floater)
"""

import argparse
import io
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

import requests
import yaml

# 修复 Windows 控制台编码问题
if sys.platform == 'win32':
    # 设置控制台输出为 UTF-8
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')


# ==================== 常量定义 ====================

# M-Team 分类映射 (开闭原则 - 易扩展)
CATEGORY_MAP = {
    # 电影类
    "401": ("SD电影", "movie"),
    "419": ("HD电影", "movie"),
    "420": ("原盘/UHD", "movie"),
    "404": ("纪录片", "movie"),
    # 剧集类
    "402": ("剧集", "tv"),
    "438": ("剧集包", "tv"),
    # 动漫
    "405": ("动漫", "anime"),
    # 综艺
    "403": ("综艺", "variety"),
    # 体育
    "418": ("体育", "sports"),
    # 成人内容
    "429": ("成人", "adult"),
}


class Config:
    """配置管理类 (单一职责原则)"""

    API_BASE = "https://api.m-team.cc/api"
    BASE_URL = "https://kp.m-team.cc"  # M-Team 站点基础 URL
    DEFAULT_HEADERS = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                     "AppleWebKit/537.36 (KHTML, like Gecko) "
                     "Chrome/120.0.0.0 Safari/537.36"
    }

    def __init__(self, config_path: str = "config.yaml"):
        """从配置文件或环境变量加载配置"""
        self.config_path = Path(config_path)
        self.api_key: str = ""
        self.proxies: Optional[Dict[str, str]] = None
        self.timeout: int = 15
        self._load_config()

    def _load_config(self) -> None:
        """加载配置文件 (开闭原则 - 易扩展配置源)"""
        if self.config_path.exists():
            with open(self.config_path, 'r', encoding='utf-8') as f:
                config_data = yaml.safe_load(f) or {}
                self.api_key = config_data.get('api_key', '')
                self.proxies = config_data.get('proxies')
                self.timeout = config_data.get('timeout', 15)
        else:
            raise FileNotFoundError(
                f"配置文件 {self.config_path} 不存在!\n"
                f"请复制 config.example.yaml 为 config.yaml 并填入你的 API Key"
            )

        if not self.api_key:
            raise ValueError("API Key 未配置!请在 config.yaml 中设置 api_key")

    def get_headers(self) -> Dict[str, str]:
        """获取带有认证的请求头"""
        headers = self.DEFAULT_HEADERS.copy()
        headers['x-api-key'] = self.api_key
        return headers


# ==================== 工具函数 ====================

def format_size(bytes_size: Any) -> str:
    """
    格式化文件大小 (KISS 原则 - 单一功能)

    Args:
        bytes_size: 字节大小

    Returns:
        格式化后的大小字符串 (如: 1.23 GB)
    """
    try:
        bytes_size = float(bytes_size)
        for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
            if bytes_size < 1024.0:
                return f"{bytes_size:.2f} {unit}"
            bytes_size /= 1024.0
        return f"{bytes_size:.2f} PB"
    except (TypeError, ValueError):
        return "未知"


def get_seeders(torrent: Dict[str, Any]) -> int:
    """
    安全获取做种人数 (防御性编程)

    Args:
        torrent: 种子数据字典

    Returns:
        做种人数,默认为 0
    """
    if 'status' in torrent and isinstance(torrent['status'], dict):
        return int(torrent['status'].get('seeders') or 0)
    return int(torrent.get('seeders') or 0)


def has_chinese_subtitle(title: str) -> bool:
    """
    判断标题是否包含中文字幕关键词 (DRY 原则 - 复用逻辑)

    Args:
        title: 种子标题

    Returns:
        是否包含中文字幕
    """
    keywords = ['中字', '简', '繁', '中英', 'chs', 'cht', 'chi', '国配', '国语']
    title_lower = title.lower()
    return any(kw in title_lower for kw in keywords)


def get_torrent_url(torrent_id: str) -> str:
    """
    生成种子页面链接 (KISS 原则 - 单一功能)

    Args:
        torrent_id: 种子 ID

    Returns:
        种子页面完整 URL
    """
    return f"{Config.BASE_URL}/detail/{torrent_id}"


# ==================== 核心查询类 ====================

class MTeamQueryClient:
    """
    M-Team API 客户端 (单一职责原则 - 仅负责查询)
    """

    def __init__(self, config: Config):
        self.config = config
        self.session = requests.Session()
        self.session.headers.update(config.get_headers())

    def search(
        self,
        keyword: str,
        category: Optional[int] = None,
        sort_field: str = "seeders",
        sort_dir: str = "desc",
        limit: int = 20
    ) -> List[Dict[str, Any]]:
        """
        搜索种子 (接口隔离原则 - 参数精简)

        Args:
            keyword: 搜索关键词
            category: 分类 ID (可选)
            sort_field: 排序字段 (seeders, size, time)
            sort_dir: 排序方向 (asc, desc)
            limit: 返回结果数量限制

        Returns:
            种子列表
        """
        payload = {"keyword": keyword, "visible": 1}

        if category:
            payload["category"] = category

        try:
            response = self.session.post(
                f"{self.config.API_BASE}/torrent/search",
                json=payload,
                proxies=self.config.proxies,
                timeout=self.config.timeout
            )
            response.raise_for_status()

            data = response.json().get('data', {}).get('data', [])
            return self._sort_results(data, sort_field, sort_dir)[:limit]

        except requests.exceptions.RequestException as e:
            print(f"❌ 网络请求失败: {e}")
            return []
        except json.JSONDecodeError:
            print("❌ 响应数据解析失败")
            return []

    def _sort_results(
        self,
        torrents: List[Dict[str, Any]],
        field: str,
        direction: str
    ) -> List[Dict[str, Any]]:
        """
        排序结果 (开闭原则 - 易扩展排序逻辑)

        Args:
            torrents: 种子列表
            field: 排序字段
            direction: 排序方向

        Returns:
            排序后的种子列表
        """
        reverse = direction.lower() == "desc"

        if field == "seeders":
            # 智能排序: 优先中文字幕,然后按做种数
            return sorted(
                torrents,
                key=lambda x: (
                    has_chinese_subtitle(x.get('name', '')),
                    get_seeders(x)
                ),
                reverse=True
            )
        elif field == "size":
            return sorted(
                torrents,
                key=lambda x: x.get('size', 0),
                reverse=reverse
            )
        else:  # time 或其他字段
            return sorted(
                torrents,
                key=lambda x: x.get('id', ''),
                reverse=reverse
            )

    def get_torrent_detail(self, torrent_id: str) -> Optional[Dict[str, Any]]:
        """
        获取种子详情 (依赖倒置原则 - 返回抽象数据结构)

        Args:
            torrent_id: 种子 ID

        Returns:
            种子详情字典,失败返回 None
        """
        try:
            response = self.session.post(
                f"{self.config.API_BASE}/torrent/detail",
                data={"id": torrent_id},
                proxies=self.config.proxies,
                timeout=self.config.timeout
            )
            response.raise_for_status()

            result = response.json()
            if result.get('code') == '0':
                return result.get('data')
            else:
                print(f"❌ 获取详情失败: {result.get('message', '未知错误')}")
                return None

        except Exception as e:
            print(f"❌ 获取种子详情出错: {e}")
            return None

    def get_download_url(self, torrent_id: str) -> Optional[str]:
        """
        生成下载链接 (单一职责 - 只生成链接不下载)

        Args:
            torrent_id: 种子 ID

        Returns:
            下载链接 URL,失败返回 None
        """
        try:
            response = self.session.post(
                f"{self.config.API_BASE}/torrent/genDlToken",
                data={"id": torrent_id},
                proxies=self.config.proxies,
                timeout=10
            )
            response.raise_for_status()

            result = response.json()
            if result.get('code') == '0':
                return result.get('data')
            else:
                print(f"❌ 生成下载链接失败: {result.get('message', '未知错误')}")
                return None

        except Exception as e:
            print(f"❌ 生成下载链接出错: {e}")
            return None

    def download_torrent(self, torrent_id: str, save_path: str = ".") -> Optional[str]:
        """
        下载种子文件到本地 (新增功能)

        Args:
            torrent_id: 种子 ID
            save_path: 保存目录路径

        Returns:
            保存的文件路径,失败返回 None
        """
        try:
            # 1. 获取下载链接
            download_url = self.get_download_url(torrent_id)
            if not download_url:
                return None

            # 2. 下载种子文件
            response = self.session.get(
                download_url,
                headers=self.config.get_headers(),
                proxies=self.config.proxies,
                timeout=15
            )
            response.raise_for_status()

            # 验证是否为有效的种子文件
            if not response.content.startswith(b'd'):
                print("❌ 下载失败：获取到的不是合法的种子文件")
                return None

            # 3. 保存到本地
            from pathlib import Path
            save_dir = Path(save_path)
            save_dir.mkdir(parents=True, exist_ok=True)

            filename = f"mteam_{torrent_id}.torrent"
            file_path = save_dir / filename

            with open(file_path, "wb") as f:
                f.write(response.content)

            return str(file_path)

        except Exception as e:
            print(f"❌ 下载种子文件失败: {e}")
            return None


# ==================== 输出格式化类 ====================

class TorrentPrinter:
    """种子信息打印器 (单一职责原则 - 只负责格式化输出)"""

    @staticmethod
    def print_table(torrents: List[Dict[str, Any]], show_index: bool = True) -> None:
        """
        以表格形式打印种子列表 (包含页面链接)

        Args:
            torrents: 种子列表
            show_index: 是否显示序号
        """
        if not torrents:
            print("📭 没有找到相关种子喵～")
            return

        print("\n" + "=" * 150)
        print(f"{'序号':<6} {'标题':<45} {'大小':<10} {'做种':<7} {'下载':<7} {'分类':<7} {'种子页面链接':<50}")
        print("=" * 150)

        for idx, torrent in enumerate(torrents, start=1):
            # 优先使用 smallDescr（中文标题），回退到 name
            small_descr = torrent.get('smallDescr', '')
            if small_descr:
                # 提取 smallDescr 中第一个 | 之前的内容（通常是中文标题）
                title = small_descr.split('|')[0].strip()[:42]
            else:
                title = torrent.get('name', '未知标题')[:42]

            size = format_size(torrent.get('size', 0))
            seeders = get_seeders(torrent)
            leechers = torrent.get('status', {}).get('leechers', 0) if isinstance(torrent.get('status'), dict) else 0
            category = torrent.get('category', '未知')
            torrent_id = torrent.get('id', '')
            torrent_url = get_torrent_url(torrent_id)

            # 中文字幕标记
            prefix = "💖" if has_chinese_subtitle(torrent.get('name', '')) else "  "
            title_display = f"{prefix} {title}"

            index_str = f"[{idx}] " if show_index else "    "

            print(
                f"{index_str:<6} {title_display:<45} {size:<10} "
                f"{seeders:<7} {leechers:<7} {category:<7} {torrent_url:<50}"
            )

        print("=" * 150 + "\n")

    @staticmethod
    def print_detail(torrent: Dict[str, Any]) -> None:
        """
        打印种子详细信息

        Args:
            torrent: 种子详情字典
        """
        print("\n" + "═" * 60)
        print(f"📦 种子详情")
        print("═" * 60)

        # 优先显示中文标题
        small_descr = torrent.get('smallDescr', '')
        if small_descr:
            print(f"标题: {small_descr}")
        print(f"文件名: {torrent.get('name', '未知')}")
        print(f"大小: {format_size(torrent.get('size', 0))}")
        print(f"分类: {torrent.get('category', '未知')}")
        print(f"ID: {torrent.get('id', '未知')}")

        if 'status' in torrent and isinstance(torrent['status'], dict):
            status = torrent['status']
            print(f"做种数: {status.get('seeders', 0)}")
            print(f"下载数: {status.get('leechers', 0)}")
            print(f"完成数: {status.get('completed', 0)}")

        if 'description' in torrent:
            print(f"\n描述:\n{torrent['description']}")

        print("═" * 60 + "\n")


# ==================== 交互式菜单类 ====================

class InteractiveMenu:
    """
    交互式菜单系统 (单一职责原则 - 只处理用户交互)
    """

    def __init__(self, client: MTeamQueryClient):
        self.client = client
        self.current_torrents: List[Dict[str, Any]] = []
        # 默认搜索设置
        self.default_category: Optional[int] = None
        self.default_sort = "seeders"
        self.default_limit = 20

    def show_welcome(self) -> None:
        """显示欢迎界面"""
        print("\n" + "🌟" * 50)
        print("🌸                                                    🌸")
        print("🌸          欢迎使用 M-Team 种子查询工具 v2.0         🌸")
        print("🌸                    by 幽浮喵 ฅ'ω'ฅ                  🌸")
        print("🌸                                                    🌸")
        print("🌟" * 50 + "\n")

    def show_main_menu(self) -> None:
        """显示主菜单"""
        print("\n" + "=" * 60)
        print("📋 主菜单")
        print("=" * 60)
        print("  1. 🔍 搜索种子")
        print("  2. 📂 按分类浏览")
        print("  3. ⚙️  默认设置")
        print("  0. 🚪 退出")
        print("=" * 60)

        # 显示当前默认设置
        cat_str = "全部" if self.default_category is None else self.default_category
        print(f"\n📌 当前默认: 分类={cat_str}, 排序={self.default_sort}, 数量={self.default_limit}")

    def show_category_menu(self) -> None:
        """显示分类菜单"""
        print("\n" + "=" * 60)
        print("📂 选择分类")
        print("=" * 60)

        # 按类型分组显示
        categories = {
            "电影 🎬": ["401", "419", "420", "404"],
            "剧集 📺": ["402", "438"],
            "动漫 🎌": ["405"],
            "综艺 🎭": ["403"],
            "体育 ⚽": ["418"],
            "成人 🔞": ["429"],  # 新增成人分类
        }

        for group, cats in categories.items():
            print(f"\n【{group}】")
            for cat_id in cats:
                if cat_id in CATEGORY_MAP:
                    name, _ = CATEGORY_MAP[cat_id]
                    print(f"  {cat_id}. {name}")

        print("\n  0. 🔙 返回主菜单")
        print("=" * 60)

    def get_user_choice(self, prompt: str = "请选择: ") -> str:
        """
        获取用户输入

        Args:
            prompt: 提示信息

        Returns:
            用户输入的内容
        """
        try:
            return input(prompt).strip()
        except (EOFError, KeyboardInterrupt):
            print("\n\n👋 用户取消操作")
            return "0"

    def search_mode(self) -> None:
        """搜索模式 (简化版 - 使用默认设置)"""
        print("\n" + "🔍" * 30)
        print("进入搜索模式")
        print("🔍" * 30)

        keyword = self.get_user_choice("\n请输入搜索关键词: ")
        if not keyword or keyword == "0":
            print("取消搜索")
            return

        # 直接使用默认设置搜索
        print(f"\n正在搜索: {keyword}...")
        print(f"使用默认设置: 分类={self.default_category or '全部'}, 排序={self.default_sort}, 数量={self.default_limit}")

        torrents = self.client.search(
            keyword=keyword,
            category=self.default_category,
            sort_field=self.default_sort,
            limit=self.default_limit
        )

        self.current_torrents = torrents
        TorrentPrinter.print_table(torrents)

        if torrents:
            self.handle_torrent_selection()

    def browse_mode(self) -> None:
        """分类浏览模式"""
        print("\n" + "📂" * 30)
        print("进入分类浏览模式")
        print("📂" * 30)

        self.show_category_menu()
        category_id = self.get_user_choice("\n请选择分类编号: ")

        if category_id == "0":
            return

        if category_id not in CATEGORY_MAP:
            print("❌ 无效的分类编号")
            return

        category_name, _ = CATEGORY_MAP[category_id]
        print(f"\n正在浏览分类: {category_name}")

        # 使用默认设置浏览该分类
        torrents = self.client.search(
            keyword="",
            category=int(category_id),
            sort_field=self.default_sort,
            limit=self.default_limit
        )

        self.current_torrents = torrents
        TorrentPrinter.print_table(torrents)

        if torrents:
            self.handle_torrent_selection()

    def handle_torrent_selection(self) -> None:
        """处理种子选择和操作"""
        while True:
            print("\n操作选项:")
            print("  1. 📥 下载种子文件")
            print("  2. 🔗 获取下载链接")
            print("  3. 📄 查看种子详情")
            print("  4. 🔍 重新搜索")
            print("  5. 📂 按分类浏览")
            print("  6. ⚙️  默认设置")
            print("  0. 🚪 退出程序")

            choice = self.get_user_choice("请选择操作: ")

            if choice == "0":
                print("\n👋 感谢使用,再见喵～ ฅ'ω'ฅ\n")
                sys.exit(0)
            elif choice == "1":
                self.download_torrent()
            elif choice == "2":
                self.get_download_link()
            elif choice == "3":
                self.view_detail()
            elif choice == "4":
                self.search_mode()
                return
            elif choice == "5":
                self.browse_mode()
                return
            elif choice == "6":
                self.settings_mode()
            else:
                print("❌ 无效的选择")

    def download_torrent(self) -> None:
        """下载种子文件"""
        if not self.current_torrents:
            print("❌ 没有可下载的种子")
            return

        index_str = self.get_user_choice("请输入要下载的种子序号: ")
        if not index_str.isdigit():
            print("❌ 无效的序号")
            return

        index = int(index_str) - 1
        if index < 0 or index >= len(self.current_torrents):
            print("❌ 序号超出范围")
            return

        torrent = self.current_torrents[index]
        torrent_id = torrent.get('id', '')
        torrent_name = torrent.get('name', '未知')

        # 询问保存路径
        save_path = self.get_user_choice("保存路径 (默认当前目录): ") or "."

        print(f"\n正在下载: {torrent_name}...")
        file_path = self.client.download_torrent(torrent_id, save_path)

        if file_path:
            print(f"✅ 下载成功!")
            print(f"📁 保存位置: {file_path}")
        else:
            print("❌ 下载失败")

    def get_download_link(self) -> None:
        """获取下载链接"""
        if not self.current_torrents:
            print("❌ 没有可选的种子")
            return

        index_str = self.get_user_choice("请输入种子序号: ")
        if not index_str.isdigit():
            print("❌ 无效的序号")
            return

        index = int(index_str) - 1
        if index < 0 or index >= len(self.current_torrents):
            print("❌ 序号超出范围")
            return

        torrent = self.current_torrents[index]
        torrent_id = torrent.get('id', '')

        url = self.client.get_download_url(torrent_id)
        if url:
            print(f"\n🔗 下载链接:\n{url}\n")
            print("💡 提示: 可以在支持的下载工具中直接使用此链接")
        else:
            print("❌ 获取下载链接失败")

    def view_detail(self) -> None:
        """查看种子详情"""
        if not self.current_torrents:
            print("❌ 没有可查看的种子")
            return

        index_str = self.get_user_choice("请输入种子序号: ")
        if not index_str.isdigit():
            print("❌ 无效的序号")
            return

        index = int(index_str) - 1
        if index < 0 or index >= len(self.current_torrents):
            print("❌ 序号超出范围")
            return

        torrent = self.current_torrents[index]
        torrent_id = torrent.get('id', '')

        detail = self.client.get_torrent_detail(torrent_id)
        if detail:
            TorrentPrinter.print_detail(detail)

    def settings_mode(self) -> None:
        """设置默认搜索参数"""
        while True:
            print("\n" + "=" * 60)
            print("⚙️  默认设置")
            print("=" * 60)
            print(f"  1. 分类筛选: {self.default_category if self.default_category else '全部'}")
            print(f"  2. 排序方式: {self.default_sort}")
            print(f"  3. 结果数量: {self.default_limit}")
            print("  0. 返回主菜单")
            print("=" * 60)

            choice = self.get_user_choice("\n请选择要修改的设置: ")

            if choice == "0":
                break
            elif choice == "1":
                self._set_default_category()
            elif choice == "2":
                self._set_default_sort()
            elif choice == "3":
                self._set_default_limit()
            else:
                print("❌ 无效的选择")

    def _set_default_category(self) -> None:
        """设置默认分类"""
        print("\n设置默认分类:")
        print("  0. 全部 (不筛选)")
        self.show_category_menu()

        category_id = self.get_user_choice("请选择分类编号: ")
        if category_id == "0":
            self.default_category = None
            print("✅ 已设置为: 全部")
        elif category_id in CATEGORY_MAP:
            self.default_category = int(category_id)
            name, _ = CATEGORY_MAP[category_id]
            print(f"✅ 已设置为: {name}")
        else:
            print("❌ 无效的分类编号")

    def _set_default_sort(self) -> None:
        """设置默认排序方式"""
        print("\n排序方式:")
        print("  1. 做种数 (seeders)")
        print("  2. 文件大小 (size)")
        print("  3. 发布时间 (time)")

        choice = self.get_user_choice("请选择排序方式 (1-3): ")
        sort_map = {"1": "seeders", "2": "size", "3": "time"}

        if choice in sort_map:
            self.default_sort = sort_map[choice]
            print(f"✅ 已设置为: {self.default_sort}")
        else:
            print("❌ 无效的选择")

    def _set_default_limit(self) -> None:
        """设置默认结果数量"""
        limit_str = self.get_user_choice(f"请输入结果数量 (当前={self.default_limit}): ")
        if limit_str.isdigit():
            self.default_limit = int(limit_str)
            print(f"✅ 已设置为: {self.default_limit}")
        else:
            print("❌ 请输入有效的数字")

    def run(self) -> None:
        """运行交互式主循环 (直接进入搜索模式)"""
        self.show_welcome()

        # 显示快捷操作提示
        print("\n💡 提示:")
        print("  - 直接输入关键词开始搜索")
        print("  - 输入 'menu' 打开主菜单")
        print("  - 输入 'browse' 按分类浏览")
        print("  - 输入 'settings' 修改默认设置")
        print("  - 输入 'exit' 或 'quit' 退出程序\n")

        # 直接进入搜索模式
        self.quick_search_mode()

    def quick_search_mode(self) -> None:
        """快速搜索模式 (启动时的默认模式)"""
        while True:
            print("\n" + "🔍" * 30)
            print("请输入搜索关键词 (或输入 help 查看命令)")
            print("🔍" * 30)

            keyword = self.get_user_choice("\n🔎 搜索: ")

            # 处理快捷命令
            if keyword.lower() in ['exit', 'quit', '0', 'q']:
                print("\n👋 感谢使用,再见喵～ ฅ'ω'ฅ\n")
                break
            elif keyword.lower() == 'menu':
                self.main_menu_mode()
                continue
            elif keyword.lower() == 'browse':
                self.browse_mode()
                continue
            elif keyword.lower() == 'settings':
                self.settings_mode()
                continue
            elif keyword.lower() == 'help':
                self.show_help()
                continue
            elif not keyword:
                continue

            # 执行搜索
            print(f"\n正在搜索: {keyword}...")
            print(f"使用默认设置: 分类={self.default_category or '全部'}, 排序={self.default_sort}, 数量={self.default_limit}")

            torrents = self.client.search(
                keyword=keyword,
                category=self.default_category,
                sort_field=self.default_sort,
                limit=self.default_limit
            )

            self.current_torrents = torrents
            TorrentPrinter.print_table(torrents)

            if torrents:
                self.handle_torrent_selection()

    def main_menu_mode(self) -> None:
        """传统主菜单模式"""
        while True:
            print("\n" + "=" * 60)
            print("📋 主菜单")
            print("=" * 60)
            print("  1. 🔍 搜索种子")
            print("  2. 📂 按分类浏览")
            print("  3. ⚙️  默认设置")
            print("  0. 🚪 退出")
            print("=" * 60)

            # 显示当前默认设置
            cat_str = "全部" if self.default_category is None else self.default_category
            print(f"\n📌 当前默认: 分类={cat_str}, 排序={self.default_sort}, 数量={self.default_limit}")

            choice = self.get_user_choice("\n请选择功能 (0-3): ")

            if choice == "0":
                print("\n👋 感谢使用,再见喵～ ฅ'ω'ฅ\n")
                sys.exit(0)
            elif choice == "1":
                self.search_mode()
                break
            elif choice == "2":
                self.browse_mode()
                break
            elif choice == "3":
                self.settings_mode()
            else:
                print("\n❌ 无效的选择,请重新输入")

    def show_help(self) -> None:
        """显示帮助信息"""
        print("\n" + "=" * 60)
        print("📖 帮助信息")
        print("=" * 60)
        print("🔍 搜索命令:")
        print("  直接输入关键词 - 开始搜索")
        print("  help         - 显示此帮助")
        print("  menu         - 打开主菜单")
        print("  browse       - 按分类浏览")
        print("  settings     - 修改默认设置")
        print("  exit/quit    - 退出程序")
        print("\n💡 搜索技巧:")
        print("  - 输入关键词后直接显示结果")
        print("  - 使用默认设置进行搜索")
        print("  - 可以在 settings 中修改默认值")
        print("=" * 60 + "\n")


# ==================== 命令行界面 ====================

def parse_arguments() -> argparse.Namespace:
    """
    解析命令行参数 (单一职责原则)

    Returns:
        解析后的参数对象
    """
    parser = argparse.ArgumentParser(
        description="M-Team 种子查询工具 - 使用 API Token 查询种子资源",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例用法:
  %(prog)s -i                        # 启动交互式模式 (推荐)
  %(prog)s search "沙丘2"            # 搜索种子
  %(prog)s search "沙丘2" -l 10      # 限制返回数量
  %(prog)s search "电影" -c 401      # 按分类搜索 (401=SD电影)
  %(prog)s detail 123456             # 查看种子详情
  %(prog)s url 123456                # 获取下载链接
  %(prog)s search "沙丘" -s size     # 按大小排序

M-Team 分类 ID 参考:
  401: SD电影    419: HD电影    420: 原盘/UHD
  402: 剧集      438: 剧集包    405: 动漫
  404: 纪录片    403: 综艺      418: 体育
        """
    )

    parser.add_argument(
        '-c', '--config',
        default='config.yaml',
        help='配置文件路径 (默认: config.yaml)'
    )

    parser.add_argument(
        '-i', '--interactive',
        action='store_true',
        help='启动交互式模式'
    )

    subparsers = parser.add_subparsers(dest='command', help='可用命令')

    # search 子命令
    search_parser = subparsers.add_parser('search', help='搜索种子')
    search_parser.add_argument('keyword', help='搜索关键词')
    search_parser.add_argument('-c', '--category', type=int, help='分类 ID')
    search_parser.add_argument('-l', '--limit', type=int, default=20, help='返回数量 (默认: 20)')
    search_parser.add_argument(
        '-s', '--sort',
        choices=['seeders', 'size', 'time'],
        default='seeders',
        help='排序字段 (默认: seeders)'
    )
    search_parser.add_argument(
        '-d', '--direction',
        choices=['asc', 'desc'],
        default='desc',
        help='排序方向 (默认: desc)'
    )

    # detail 子命令
    detail_parser = subparsers.add_parser('detail', help='查看种子详情')
    detail_parser.add_argument('torrent_id', help='种子 ID')

    # url 子命令
    url_parser = subparsers.add_parser('url', help='获取下载链接')
    url_parser.add_argument('torrent_id', help='种子 ID')

    return parser.parse_args()


def main() -> int:
    """
    主函数 (依赖倒置原则 - 依赖抽象而非具体实现)

    Returns:
        退出码 (0=成功, 1=失败)
    """
    try:
        args = parse_arguments()

        # 加载配置
        config = Config(args.config)

        # 创建客户端
        client = MTeamQueryClient(config)

        # 交互式模式
        if args.interactive or not args.command:
            menu = InteractiveMenu(client)
            menu.run()
            return 0

        # 命令行模式
        if args.command == 'search':
            torrents = client.search(
                keyword=args.keyword,
                category=args.category,
                sort_field=args.sort,
                sort_dir=args.direction,
                limit=args.limit
            )
            TorrentPrinter.print_table(torrents)

            if torrents:
                print(f"✅ 找到 {len(torrents)} 个相关种子")

        elif args.command == 'detail':
            detail = client.get_torrent_detail(args.torrent_id)
            if detail:
                TorrentPrinter.print_detail(detail)
            else:
                return 1

        elif args.command == 'url':
            url = client.get_download_url(args.torrent_id)
            if url:
                print(f"\n🔗 下载链接:\n{url}\n")
            else:
                return 1

        return 0

    except FileNotFoundError as e:
        print(f"❌ {e}")
        return 1
    except ValueError as e:
        print(f"❌ 配置错误: {e}")
        return 1
    except KeyboardInterrupt:
        print("\n\n👋 用户取消操作")
        return 0
    except Exception as e:
        print(f"❌ 未知错误: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
