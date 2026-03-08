# 🚀 快速打包指南

## Windows 用户

### 方法一：双击运行（最简单）

1. 双击 `build.bat`
2. 等待打包完成
3. 在 `dist` 目录找到 `mteam-query.exe`

### 方法二：命令行运行

```bash
# 1. 安装打包依赖
pip install -r build-requirements.txt

# 2. 运行打包脚本
python build.py
```

## Linux/Mac 用户

```bash
# 1. 给脚本添加执行权限
chmod +x build.sh

# 2. 运行打包脚本
./build.sh
```

或者：

```bash
# 1. 安装打包依赖
pip3 install -r build-requirements.txt

# 2. 运行打包脚本
python3 build.py
```

## 打包完成后

### 目录结构

```
m-team爬虫/
├── dist/
│   ├── mteam-query.exe    # Windows 可执行文件
│   └── config.yaml         # 配置文件
├── build/                  # 临时文件（可删除）
└── mteam-query.spec        # 打包配置（可删除）
```

### 使用打包后的程序

1. **复制文件到目标位置**
   ```
   你的文件夹/
   ├── mteam-query.exe
   └── config.yaml
   ```

2. **配置 API Key**
   - 编辑 `config.yaml`
   - 填入你的 M-Team API Key

3. **运行程序**
   ```bash
   # Windows
   mteam-query.exe

   # Linux/Mac
   ./mteam-query
   ```

## 清理临时文件

打包脚本会询问是否清理临时文件，包括：
- `build/` 目录
- `mteam-query.spec` 文件

也可以手动删除：

```bash
# Windows
rmdir /s build
del mteam-query.spec

# Linux/Mac
rm -rf build mteam-query.spec
```

## 常见问题

### Q: 第一次运行很慢？
A: 单文件打包的程序首次运行需要解压，需要几秒钟，正常现象。

### Q: 杀毒软件报警？
A: PyInstaller 打包的程序可能被误报，添加到白名单即可。

### Q: 提示找不到配置文件？
A: 确保 `config.yaml` 和可执行文件在同一目录。

### Q: 如何更新程序？
A: 重新运行打包脚本，生成新的可执行文件。

## 详细文档

查看 [BUILD.md](BUILD.md) 获取更多详细信息和高级选项。

---

**Author:** 幽浮喵 (Floater)
**Date:** 2025-03-08
