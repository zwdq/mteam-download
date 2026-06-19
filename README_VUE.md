# Vue Frontend Notes

这个项目现在额外带了一套 `Vue 3 + Vite` 前端实现，原有的 `Python` / `Rust` 代码都保留未删。

## 新增目录

- `package.json`
- `vite.config.js`
- `index.html`
- `src/`

## 运行方式

```bash
npm install
npm run dev
```

## 说明

- 原来的 `mteam_query.py`、`cargo/`、`config.yaml` 继续保留
- 新前端把主要查询、详情、下载链接流程迁到了浏览器侧的 `Vue + JS`
- 如果目标站点有 CORS 或网络限制，后续可以再补代理层
