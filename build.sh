#!/bin/bash
# M-Team 种子查询工具 - 打包脚本 (Linux/Mac)

set -e

echo ""
echo "===================================================="
echo "   M-Team 种子查询工具 - 快速打包脚本"
echo "===================================================="
echo ""

# 检查 Python 是否安装
if ! command -v python3 &> /dev/null; then
    echo "❌ 错误: 未检测到 Python3"
    echo "请先安装 Python: https://www.python.org/"
    exit 1
fi

echo "✅ 检测到 Python 环境"
python3 --version
echo ""

# 运行打包脚本
echo "🔨 开始打包..."
echo ""
python3 build.py

# 检查打包结果
if [ -f "dist/mteam-query" ] || [ -f "dist/mteam-query.exe" ]; then
    echo ""
    echo "===================================================="
    echo "🎉 打包成功！"
    echo "===================================================="
    echo ""
    echo "📦 可执行文件位置: dist/"
    echo "📄 配置文件位置: config.yaml"
    echo ""
    echo "💡 使用方法:"
    echo "   1. 将 dist/ 中的可执行文件复制到任意目录"
    echo "   2. 同时复制 config.yaml 到同一目录"
    echo "   3. 运行程序:"
    echo "      - Linux: ./mteam-query"
    echo "      - Mac: ./mteam-query"
    echo ""
    echo "===================================================="
else
    echo ""
    echo "===================================================="
    echo "❌ 打包失败"
    echo "===================================================="
fi

echo ""
echo "按 Ctrl+C 退出或等待3秒自动退出..."
sleep 3
