# Windows 打包说明

## 方法一：直接运行 Python 脚本（推荐）

打开命令提示符（CMD）或 PowerShell，执行：

```bash
python build.py
```

## 方法二：双击批处理文件

双击 `build.bat` 文件即可。

**注意**：批处理文件使用英文界面以避免编码问题。

## 如果批处理文件乱码

如果看到 `'xx' 不是内部或外部命令` 这样的错误，说明是编码问题。

**解决方法**：
1. 直接使用命令：`python build.py`
2. 或在PowerShell中运行 `python build.py`

## 完整步骤

1. 打开CMD或PowerShell
2. 进入项目目录：
   ```bash
   cd "c:\Users\zhuwu\Downloads\m-team爬虫"
   ```
3. 运行打包脚本：
   ```bash
   python build.py
   ```
4. 等待完成
5. 在 `dist` 目录找到 `mteam-query.exe`

## 常见问题

### Q: 提示找不到 Python？
A: 确保 Python 已安装并添加到系统 PATH。

### Q: 批处理文件乱码？
A: 直接使用 `python build.py` 命令。

### Q: 打包失败？
A: 检查是否安装了所有依赖：
   ```bash
   pip install -r build-requirements.txt
   ```

## 详细文档

查看 [QUICKSTART_BUILD.md](QUICKSTART_BUILD.md) 获取更多信息。
