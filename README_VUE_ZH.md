# M-Team Vue 前端说明

这个目录在保留原有 `Python` / `Rust` 代码的前提下，新增了一套 `Vue 3 + Vite` 前端。

## 保留的旧代码

- `mteam_query.py`
- `cargo/`
- `config.yaml`
- 原有构建脚本与文档

## 新增前端

- `package.json`
- `vite.config.js`
- `index.html`
- `src/App.vue`
- `src/lib/mteam-api.js`

## 开发运行

```bash
npm install
npm run dev
```

默认开发地址：

- `http://127.0.0.1:5174`

## 现在这版做了什么

- 用 Vue 页面重做了搜索、详情、下载链接三个主要流程
- API Key、站点地址等配置会保存在浏览器本地
- 开发模式默认通过 `/mteam-api` 走 Vite 代理，尽量减少 CORS 问题

## 还需要注意

- 这仍然依赖 M-Team API Key
- 生产环境建议再配一个正式反向代理，不要直接裸连第三方 API
