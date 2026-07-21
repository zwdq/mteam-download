// M-Team API 客户端
// 与 Python 版 mteam_query.py 对齐

const MTEAM_API = "https://api.m-team.cc/api";

/**
 * 解析 API Base，开发环境用 Vite 代理，生产环境用 Cloudflare Worker 代理
 */
function resolveApiBase(apiBase) {
  const value = String(apiBase || "").trim();
  if (!value) {
    return import.meta.env.DEV ? "/mteam-api" : "https://mteam-proxy.zhuwudaoqin.workers.dev/mteam-api";
  }

  // 只在开发环境下把直连 URL 重写为代理路径
  if (import.meta.env.DEV && (
    value === "https://api.m-team.cc/api" ||
    value === "http://api.m-team.cc/api" ||
    value === "https://api.m-team.cc" ||
    value === "http://api.m-team.cc"
  )) {
    return "/mteam-api";
  }

  return value.replace(/\/$/, "");
}

/**
 * 统一请求：发送请求并解析 JSON
 */
async function requestJson(url, options) {
  const response = await fetch(url, options);
  const text = await response.text();
  let payload = {};

  try {
    payload = text ? JSON.parse(text) : {};
  } catch {
    if (text.includes("Invalid CORS request")) {
      throw new Error(
        "请求被目标站点拦截了。请确认 API 地址使用的是代理地址，而非直连 https://api.m-team.cc/api"
      );
    }
    throw new Error(`返回内容不是合法 JSON：${text.slice(0, 160)}`);
  }

  if (!response.ok) {
    throw new Error(payload?.message || `请求失败：${response.status}`);
  }

  // M-Team API 约定：code=0 成功，非 0 失败
  if (payload?.code !== undefined && String(payload.code) !== "0") {
    throw new Error(payload.message || `API 错误：${payload.code}`);
  }

  return payload;
}

/**
 * POST JSON 请求
 */
async function postJson(url, body, apiKey) {
  return requestJson(url, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-api-key": apiKey,
    },
    body: JSON.stringify(body),
  });
}

/**
 * POST form 请求（detail / genDlToken 用 form-urlencoded）
 */
async function postForm(url, formData, apiKey) {
  const body = new URLSearchParams();
  for (const [key, value] of Object.entries(formData)) {
    body.set(key, String(value));
  }
  return requestJson(url, {
    method: "POST",
    headers: {
      "content-type": "application/x-www-form-urlencoded; charset=UTF-8",
      "x-api-key": apiKey,
    },
    body: body.toString(),
  });
}

/**
 * 搜索种子
 * 与 Python 版 search() 对齐：
 *   - 加分页参数 pageNumber / pageSize（让 API 返回足够数据）
 *   - 排序字段传给 API（sortField + sortOrder）
 *   - 返回数据直接取 data.data（与 Python 一致）
 */
export async function searchTorrents(settings, filters) {
  const apiBase = resolveApiBase(settings.apiBase);

  // 排序字段映射：前端 → API
  const sortFieldMap = {
    seeders: "seeders",
    size: "size",
    time: "created",
  };
  const sortField = sortFieldMap[filters.sortField] || "seeders";
  const sortOrder = filters.sortDirection === "asc" ? "asc" : "desc";

  const limit = Number(filters.limit || 20);

  // 与 Python 版一致的 payload，加上分页
  const payload = {
    keyword: filters.keyword,
    visible: 1,
    pageNumber: 1,
    pageSize: limit,
    sortField: sortField,
    sortOrder: sortOrder,
  };

  // 有分类时加上 category
  if (filters.category) {
    payload.category = Number(filters.category);
  }

  const data = await postJson(`${apiBase}/torrent/search`, payload, settings.apiKey);

  // 与 Python 版一致：response.json().get('data', {}).get('data', [])
  const items = data?.data?.data || [];

  // 前端兜底排序（API 不支持时仍有效）
  items.sort((a, b) => {
    const key = filters.sortField;
    const av = key === "seeders" ? Number(a?.status?.seeders || a?.seeders || 0) : Number(a?.[key] || 0);
    const bv = key === "seeders" ? Number(b?.status?.seeders || b?.seeders || 0) : Number(b?.[key] || 0);
    return filters.sortDirection === "asc" ? av - bv : bv - av;
  });

  return items.slice(0, limit);
}

/**
 * 获取种子详情
 */
export async function fetchTorrentDetail(settings, torrentId) {
  const apiBase = resolveApiBase(settings.apiBase);
  const data = await postForm(
    `${apiBase}/torrent/detail`,
    { id: String(torrentId) },
    settings.apiKey
  );
  // 与 Python 版一致：response.json().get('data', {})
  return data?.data || null;
}

/**
 * 生成下载链接
 * 注意：下载链接必须直连 M-Team，不走代理
 */
export async function generateDownloadUrl(settings, torrentId) {
  const apiBase = resolveApiBase(settings.apiBase);
  const data = await postForm(
    `${apiBase}/torrent/genDlToken`,
    { id: String(torrentId) },
    settings.apiKey
  );

  const token = data?.data;
  if (!token) {
    throw new Error("没有拿到下载 token");
  }

  // 如果返回的是完整 URL，直接用
  if (typeof token === "string" && /^https?:\/\//i.test(token)) {
    return token;
  }

  // 下载链接直连 M-Team 原始 API，不走代理
  return `${MTEAM_API}/torrent/download/${token}`;
}
