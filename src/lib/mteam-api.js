function normalizeListPayload(payload) {
  if (Array.isArray(payload?.data?.data)) {
    return payload.data.data;
  }
  if (Array.isArray(payload?.data?.data?.data)) {
    return payload.data.data.data;
  }
  if (Array.isArray(payload?.data?.data?.dataItems)) {
    return payload.data.data.dataItems;
  }
  if (Array.isArray(payload?.data?.dataItems)) {
    return payload.data.dataItems;
  }
  if (Array.isArray(payload?.data)) {
    return payload.data;
  }
  return [];
}

function normalizeDetailPayload(payload) {
  return payload?.data || payload?.data?.data || null;
}

function resolveApiBase(apiBase) {
  const value = String(apiBase || "").trim();
  if (!value) {
    return "/mteam-api";
  }

  if (
    value === "https://api.m-team.cc/api" ||
    value === "http://api.m-team.cc/api" ||
    value === "https://api.m-team.cc" ||
    value === "http://api.m-team.cc"
  ) {
    return "/mteam-api";
  }

  return value.replace(/\/$/, "");
}

async function requestJson(url, options) {
  const response = await fetch(url, options);
  const text = await response.text();
  let payload = {};

  try {
    payload = text ? JSON.parse(text) : {};
  } catch {
    if (text.includes("Invalid CORS request")) {
      throw new Error(
        "请求被目标站点拦截了。请确认当前 API Base 使用的是 /mteam-api，而不是直接填 https://api.m-team.cc/api"
      );
    }
    throw new Error(`返回内容不是合法 JSON：${text.slice(0, 160)}`);
  }

  if (!response.ok) {
    throw new Error(payload?.message || `请求失败：${response.status}`);
  }

  if (payload?.code !== undefined && String(payload.code) !== "0") {
    throw new Error(payload.message || `API 错误：${payload.code}`);
  }

  return payload;
}

async function requestFormJson(url, formData, apiKey) {
  const body = new URLSearchParams();

  for (const [key, value] of Object.entries(formData)) {
    body.set(key, String(value));
  }

  return requestJson(url, {
    method: "POST",
    headers: {
      "content-type": "application/x-www-form-urlencoded; charset=UTF-8",
      "x-api-key": apiKey
    },
    body: body.toString()
  });
}

export async function searchTorrents(settings, filters) {
  const apiBase = resolveApiBase(settings.apiBase);
  const payload = filters.category
    ? { keyword: filters.keyword, visible: 1, category: Number(filters.category) }
    : { keyword: filters.keyword, visible: 1 };

  const data = await requestJson(`${apiBase}/torrent/search`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-api-key": settings.apiKey
    },
    body: JSON.stringify(payload)
  });

  const items = normalizeListPayload(data);
  return items
    .slice()
    .sort((a, b) => {
      const key = filters.sortField;
      const av = key === "seeders" ? Number(a?.status?.seeders || a?.seeders || 0) : Number(a?.[key] || 0);
      const bv = key === "seeders" ? Number(b?.status?.seeders || b?.seeders || 0) : Number(b?.[key] || 0);
      return filters.sortDirection === "asc" ? av - bv : bv - av;
    })
    .slice(0, Number(filters.limit || 20));
}

export async function fetchTorrentDetail(settings, torrentId) {
  const apiBase = resolveApiBase(settings.apiBase);
  const data = await requestFormJson(
    `${apiBase}/torrent/detail`,
    { id: String(torrentId) },
    settings.apiKey
  );

  return normalizeDetailPayload(data);
}

export async function generateDownloadUrl(settings, torrentId) {
  const apiBase = resolveApiBase(settings.apiBase);
  const data = await requestFormJson(
    `${apiBase}/torrent/genDlToken`,
    { id: String(torrentId) },
    settings.apiKey
  );

  const token = data?.data;
  if (!token) {
    throw new Error("没有拿到下载 token");
  }

  if (typeof token === "string" && /^https?:\/\//i.test(token)) {
    return token;
  }

  return `${apiBase}/torrent/download/${token}`;
}
