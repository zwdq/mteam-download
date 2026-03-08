#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
M-Team 种子查询工具打包脚本
使用 PyInstaller 将项目打包成单文件可执行程序
Author: 幽浮喵 (Floater)
"""

import os
import sys
import subprocess
from pathlib import Path


def check_pyinstaller():
    """检查 PyInstaller 是否已安装"""
    try:
        import PyInstaller
        return True
    except ImportError:
        return False


def install_pyinstaller():
    """安装 PyInstaller"""
    print("📦 正在安装 PyInstaller...")
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "pyinstaller"])
        print("✅ PyInstaller 安装成功")
        return True
    except subprocess.CalledProcessError:
        print("❌ PyInstaller 安装失败")
        return False


def build_executable():
    """构建可执行文件"""
    print("\n" + "🔨" * 50)
    print("开始打包 M-Team 种子查询工具")
    print("🔨" * 50 + "\n")

    # 确保在项目根目录
    project_root = Path(__file__).parent
    os.chdir(project_root)

    # PyInstaller 命令参数
    pyinstaller_args = [
        "pyinstaller",
        "--name=mteam-query",           # 可执行文件名称
        "--onefile",                     # 打包成单文件
        "--console",                     # 显示控制台窗口（交互式程序需要）
        "--icon=NONE",                   # 图标（如果有的话可以指定）
        "--add-data=config.yaml:.",      # 包含配置文件
        "--hidden-import=requests",      # 显式导入 requests
        "--hidden-import=yaml",          # 显式导入 yaml
        "--collect-all=requests",        # 收集 requests 的所有依赖
        "--collect-all=urllib3",         # 收集 urllib3 的所有依赖
        "mteam_query.py",                # 主脚本
    ]

    print("📋 打包配置:")
    print("  - 名称: mteam-query")
    print("  - 模式: 单文件 (--onefile)")
    print("  - 包含: config.yaml")
    print("  - 隐式导入: requests, yaml, urllib3\n")

    try:
        # 执行打包命令
        result = subprocess.run(pyinstaller_args, capture_output=True, text=True)

        if result.returncode == 0:
            print("\n✅ 打包成功！\n")

            # 显示输出文件位置
            dist_dir = project_root / "dist"
            exe_file = dist_dir / "mteam-query.exe"
            if exe_file.exists():
                file_size = exe_file.stat().st_size / (1024 * 1024)  # MB
                print(f"📦 可执行文件: {exe_file}")
                print(f"📏 文件大小: {file_size:.2f} MB\n")

                # 复制配置文件到 dist 目录
                import shutil
                config_dest = dist_dir / "config.yaml"
                if not config_dest.exists():
                    shutil.copy("config.yaml", config_dest)
                    print(f"✅ 配置文件已复制到: {config_dest}\n")

                print("=" * 50)
                print("🎉 打包完成！")
                print("=" * 50)
                print(f"\n💡 使用说明:")
                print(f"  1. 将 {exe_file} 和 config.yaml 放在同一目录")
                print(f"  2. 双击 mteam-query.exe 运行")
                print(f"  3. 或在命令行中: mteam-query.exe\n")

                return True
        else:
            print("\n❌ 打包失败！\n")
            print("错误输出:")
            print(result.stderr)
            return False

    except FileNotFoundError:
        print("\n❌ 找不到 PyInstaller 命令")
        print("请先安装: pip install pyinstaller")
        return False
    except Exception as e:
        print(f"\n❌ 打包过程出错: {e}")
        return False


def clean_build_files():
    """清理打包生成的临时文件"""
    print("\n🧹 清理临时文件...")

    project_root = Path(__file__).parent
    dirs_to_remove = ["build", "spec"]
    files_to_remove = ["mteam-query.spec"]

    for dir_name in dirs_to_remove:
        dir_path = project_root / dir_name
        if dir_path.exists():
            import shutil
            shutil.rmtree(dir_path)
            print(f"  ✓ 删除目录: {dir_name}")

    for file_name in files_to_remove:
        file_path = project_root / file_name
        if file_path.exists():
            file_path.unlink()
            print(f"  ✓ 删除文件: {file_name}")

    print("✅ 清理完成\n")


def main():
    """主函数"""
    print("\n" + "🌟" * 50)
    print("🌸                                                    🌸")
    print("🌸        M-Team 种子查询工具 - 打包脚本 v1.0        🌸")
    print("🌸                    by 幽浮喵 ฅ'ω'ฅ                  🌸")
    print("🌸                                                    🌸")
    print("🌟" * 50 + "\n")

    # 检查并安装 PyInstaller
    if not check_pyinstaller():
        print("⚠️  未检测到 PyInstaller")
        if input("是否立即安装? (y/n): ").strip().lower() == 'y':
            if not install_pyinstaller():
                print("\n❌ 无法继续打包，请手动安装 PyInstaller")
                print("安装命令: pip install pyinstaller\n")
                sys.exit(1)
        else:
            print("\n❌ 打包需要 PyInstaller")
            sys.exit(1)

    # 执行打包
    if build_executable():
        # 询问是否清理临时文件
        if input("\n是否清理临时文件? (y/n): ").strip().lower() == 'y':
            clean_build_files()

        print("\n👋 打包流程结束，再见喵～ ฅ'ω'ฅ\n")
    else:
        print("\n❌ 打包失败")
        sys.exit(1)


if __name__ == "__main__":
    main()
