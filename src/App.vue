<script setup>
import { computed, reactive, ref, watch } from "vue";
import { categories, sortOptions } from "./lib/categories";
import { detailUrl, formatSize, hasChineseSubtitle } from "./lib/format";
import { loadSettings, saveSettings } from "./lib/storage";
import { fetchTorrentDetail, generateDownloadUrl, searchTorrents } from "./lib/mteam-api";

const saved = loadSettings();

const settings = reactive({
  apiKey: saved.apiKey || "",
  apiBase: saved.apiBase || "/mteam-api",
  siteBase: saved.siteBase || "https://kp.m-team.cc"
});

if (
  settings.apiBase === "https://api.m-team.cc/api" ||
  settings.apiBase === "http://api.m-team.cc/api" ||
  settings.apiBase === "https://api.m-team.cc" ||
  settings.apiBase === "http://api.m-team.cc"
) {
  settings.apiBase = "/mteam-api";
}

const filters = reactive({
  keyword: "",
  category: "",
  sortField: "seeders",
  sortDirection: "desc",
  limit: 20
});

const items = ref([]);
const selected = ref(null);
const detail = ref(null);
const downloadUrl = ref("");
const loading = ref(false);
const detailLoading = ref(false);
const errorMessage = ref("");
const statusMessage = ref("填写 API Key 后即可搜索资源。");

watch(
  settings,
  () => saveSettings(settings),
  { deep: true }
);

const canSearch = computed(() => Boolean(settings.apiKey.trim() && filters.keyword.trim()));
const selectedDetailUrl = computed(() =>
  selected.value ? detailUrl(settings.siteBase, selected.value.id) : ""
);

async function handleSearch() {
  if (!canSearch.value) {
    errorMessage.value = "请先填写 API Key 和搜索关键词。";
    return;
  }

  loading.value = true;
  errorMessage.value = "";
  statusMessage.value = "正在搜索资源...";
  detail.value = null;
  selected.value = null;
  downloadUrl.value = "";

  try {
    items.value = await searchTorrents(settings, filters);
    statusMessage.value = `搜索完成，共拿到 ${items.value.length} 条结果。`;
  } catch (error) {
    items.value = [];
    errorMessage.value = error instanceof Error ? error.message : String(error);
    statusMessage.value = "搜索失败。";
  } finally {
    loading.value = false;
  }
}

async function inspectTorrent(item) {
  selected.value = item;
  detail.value = null;
  downloadUrl.value = "";
  detailLoading.value = true;
  errorMessage.value = "";
  statusMessage.value = `正在读取 ${item.name} 的详情...`;

  try {
    detail.value = await fetchTorrentDetail(settings, item.id);
    statusMessage.value = "详情已更新。";
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
    statusMessage.value = "详情读取失败。";
  } finally {
    detailLoading.value = false;
  }
}

async function createDownloadUrl(item) {
  selected.value = item;
  detail.value = null;
  downloadUrl.value = "";
  detailLoading.value = true;
  errorMessage.value = "";
  statusMessage.value = `正在生成 ${item.name} 的下载链接...`;

  try {
    downloadUrl.value = await generateDownloadUrl(settings, item.id);
    statusMessage.value = "下载链接已生成。";
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
    statusMessage.value = "生成下载链接失败。";
  } finally {
    detailLoading.value = false;
  }
}
</script>

<template>
  <div class="page-shell">
    <aside class="settings-panel">
      <div>
        <p class="eyebrow">M-Team Search</p>
        <h1>M-Team 资源查询</h1>
        <p class="muted">输入 API Key，按关键词、分类和排序方式查询资源。</p>
      </div>

      <label class="field">
        <span>API Key</span>
        <input v-model="settings.apiKey" type="password" placeholder="填写你的 M-Team API Key" />
      </label>

      <label class="field">
        <span>API 地址</span>
        <input v-model="settings.apiBase" type="text" />
      </label>

      <button class="secondary-button" type="button" @click="settings.apiBase = '/mteam-api'">
        恢复默认地址
      </button>

      <label class="field">
        <span>站点地址</span>
        <input v-model="settings.siteBase" type="text" />
      </label>

      <div class="tip-card">
        <strong>提示</strong>
        <p>API Key 仅保存在当前浏览器中。若无法请求，请先恢复默认地址后重试。</p>
      </div>
    </aside>

    <main class="workspace">
      <section class="toolbar">
        <label class="field grow">
          <span>关键词</span>
          <input v-model="filters.keyword" type="text" placeholder="例如：沙丘2" @keyup.enter="handleSearch" />
        </label>

        <label class="field">
          <span>分类</span>
          <select v-model="filters.category">
            <option v-for="category in categories" :key="category.id" :value="category.id">
              {{ category.label }}
            </option>
          </select>
        </label>

        <label class="field">
          <span>排序</span>
          <select v-model="filters.sortField">
            <option v-for="option in sortOptions" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </label>

        <label class="field">
          <span>方向</span>
          <select v-model="filters.sortDirection">
            <option value="desc">降序</option>
            <option value="asc">升序</option>
          </select>
        </label>

        <label class="field short">
          <span>条数</span>
          <input v-model.number="filters.limit" type="number" min="1" max="100" />
        </label>

        <button class="primary-button" :disabled="loading" @click="handleSearch">
          {{ loading ? "搜索中..." : "开始搜索" }}
        </button>
      </section>

      <section class="status-bar">
        <span>{{ statusMessage }}</span>
        <span v-if="errorMessage" class="error-text">{{ errorMessage }}</span>
      </section>

      <section class="content-grid">
        <div class="results-panel">
          <div class="panel-header">
            <h2>搜索结果</h2>
            <span>{{ items.length }} 条</span>
          </div>

          <div v-if="!items.length" class="empty-state">
            还没有结果，先搜一把试试。
          </div>

          <article
            v-for="item in items"
            :key="item.id"
            class="result-card"
            :class="{ active: selected && selected.id === item.id }"
          >
            <div class="result-main">
              <div class="title-row">
                <span v-if="hasChineseSubtitle(item.name, item.smallDescr)" class="badge">中字</span>
                <h3>{{ item.name }}</h3>
              </div>
              <p class="muted">{{ item.smallDescr || "无副标题" }}</p>
              <div class="meta-row">
                <span>大小 {{ formatSize(item.size) }}</span>
                <span>做种 {{ item.status?.seeders ?? item.seeders ?? 0 }}</span>
                <span>下载 {{ item.status?.leechers ?? 0 }}</span>
                <span>ID {{ item.id }}</span>
              </div>
            </div>

            <div class="action-row">
              <button @click="inspectTorrent(item)">看详情</button>
              <button @click="createDownloadUrl(item)">取链接</button>
              <a :href="detailUrl(settings.siteBase, item.id)" target="_blank" rel="noreferrer">打开页面</a>
            </div>
          </article>
        </div>

        <div class="detail-panel">
          <div class="panel-header">
            <h2>详情面板</h2>
            <span v-if="selected">{{ selected.id }}</span>
          </div>

          <div v-if="detailLoading" class="empty-state">处理中...</div>

          <div v-else-if="downloadUrl" class="detail-block">
            <h3>下载链接</h3>
            <textarea readonly :value="downloadUrl"></textarea>
            <a class="inline-link" :href="downloadUrl" target="_blank" rel="noreferrer">直接打开下载链接</a>
          </div>

          <div v-else-if="detail" class="detail-block">
            <h3>{{ detail.name }}</h3>
            <p class="muted">{{ detail.smallDescr || "无副标题" }}</p>
            <div class="detail-grid">
              <span>大小</span>
              <strong>{{ formatSize(detail.size) }}</strong>
              <span>分类</span>
              <strong>{{ detail.category }}</strong>
              <span>做种</span>
              <strong>{{ detail.status?.seeders ?? 0 }}</strong>
              <span>下载</span>
              <strong>{{ detail.status?.leechers ?? 0 }}</strong>
              <span>完成</span>
              <strong>{{ detail.status?.completed ?? 0 }}</strong>
            </div>
            <a class="inline-link" :href="selectedDetailUrl" target="_blank" rel="noreferrer">在站点打开详情页</a>
            <div v-if="detail.description" class="description-box">{{ detail.description }}</div>
          </div>

          <div v-else class="empty-state">
            选一条记录后，这里会显示详情或者下载链接。
          </div>
        </div>
      </section>
    </main>
  </div>
</template>
