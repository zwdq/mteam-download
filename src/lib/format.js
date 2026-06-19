export function formatSize(size) {
  const value = Number(size || 0);
  if (!Number.isFinite(value) || value <= 0) {
    return "0 B";
  }

  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let current = value;
  let unitIndex = 0;

  while (current >= 1024 && unitIndex < units.length - 1) {
    current /= 1024;
    unitIndex += 1;
  }

  return `${current.toFixed(2)} ${units[unitIndex]}`;
}

export function hasChineseSubtitle(name = "", smallDescr = "") {
  const text = `${name} ${smallDescr}`.toLowerCase();
  const keywords = ["中字", "中英", "国配", "国语", "简繁", "chs", "cht", "chi"];
  return keywords.some((keyword) => text.includes(keyword));
}

export function detailUrl(baseUrl, id) {
  return `${baseUrl.replace(/\/$/, "")}/detail/${id}`;
}
