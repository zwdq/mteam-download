# 📦 M-Team 种子查询工具 - 打包指南

## 快速开始

### 方法一：使用打包脚本（推荐）

```bash
# 运行打包脚本
python build.py
```

脚本会自动：
1. 检查并安装 PyInstaller
2. 执行打包
3. 生成单文件可执行程序
4. 可选清理临时文件

### 方法二：手动打包

```bash
# 1. 安装 PyInstaller
pip install pyinstaller

# 2. 执行打包命令
pyinstaller --name=mteam-query --onefile --console \
  --add-data=config.yaml:. \
  --hidden-import=requests \
  --hidden-import=yaml \
  --collect-all=requests \
  --collect-all=urllib3 \
  mteam_query.py
```

## 打包配置说明

| 参数 | 说明 |
|------|------|
| `--name=mteam-query` | 可执行文件名称 |
| `--onefile` | 打包成单文件 |
| `--console` | 显示控制台窗口（交互式程序需要）|
| `--add-data=config.yaml:.` | 包含配置文件 |
| `--hidden-import=requests` | 显式导入 requests |
| `--hidden-import=yaml` | 显式导入 yaml |
| `--collect-all=requests` | 收集 requests 的所有依赖 |
| `--collect-all=urllib3` | 收集 urllib3 的所有依赖 |

## 输出文件

打包完成后，会在 `dist` 目录下生成：

```
dist/
├── mteam-query.exe    # 主程序（单文件可执行）
└── config.yaml         # 配置文件
```

## 使用说明

### Windows 用户

1. **复制文件**
   - 将 `mteam-query.exe` 和 `config.yaml` 复制到同一目录

2. **运行程序**
   ```bash
   # 双击运行
   mteam-query.exe

   # 或命令行运行
   mteam-query.exe
   ```

3. **配置 API Key**
   - 编辑 `config.yaml`，填入你的 M-Team API Key
   - 确保代理配置正确（如果需要）

### Linux/Mac 用户

```bash
# 给执行权限
chmod +x dist/mteam-query

# 运行
./dist/mteam-query
```

## 注意事项

### 1. 配置文件

打包后的程序需要 `config.yaml` 文件在同一目录：

```
your-folder/
├── mteam-query.exe
└── config.yaml
```

### 2. 杀毒软件

某些杀毒软件可能会误报 PyInstaller 打包的程序：
- 可以添加到白名单
- 或使用虚拟机测试

### 3. 文件大小

单文件打包后的程序较大（约 20-30 MB），这是因为：
- 包含了 Python 解释器
- 包含了所有依赖库

### 4. 启动速度

单文件程序首次启动时会解压到临时目录，可能需要几秒钟。

## 常见问题

### Q: 打包后运行提示找不到配置文件？

A: 确保 `config.yaml` 和可执行文件在同一目录。

### Q: 打包后程序无法运行？

A:
1. 检查是否使用了 `--console` 参数
2. 查看错误信息
3. 尝试在命令行运行以查看详细错误

### Q: 如何减小文件大小？

A:
- 使用 `--exclude-module` 排除不需要的模块
- 或者使用 `--onedir` 打包成目录模式

### Q: 能否添加图标？

A: 可以准备一个 `.ico` 文件（Windows）或 `.icns` 文件（Mac），然后：

```bash
pyinstaller --icon=your_icon.ico ...
```

## 高级选项

### 打包成目录模式（更快启动）

```bash
pyinstaller --name=mteam-query --onedir --console \
  --add-data=config.yaml:. \
  --hidden-import=requests \
  mteam_query.py
```

### 添加版本信息

编辑 `mteam-query.spec` 文件，添加版本信息：

```python
exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='mteam-query',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    version='version.txt'  # Windows 版本信息
)
```

## 技术支持

如遇到问题，请：
1. 查看错误信息
2. 检查 PyInstaller 版本：`pyinstaller --version`
3. 尝试更新依赖：`pip install --upgrade pyinstaller`

---

**Author:** 幽浮喵 (Floater)
**Version:** 1.0.0
**Date:** 2025-03-08
