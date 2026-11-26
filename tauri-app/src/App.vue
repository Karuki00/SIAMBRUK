<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted } from "vue";
import packageInfo from "../package.json";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { RouterLink, RouterView, useRoute } from "vue-router";
import { BarChart3, Boxes, ChevronRight, ClipboardList, History, LayoutDashboard, Package, Settings, ShoppingCart, Sparkles, Store, Warehouse, Map } from "lucide-vue-next";

const route = useRoute();
const navigation = [
  { label: "Dashboard", to: "/", icon: LayoutDashboard },
  { label: "Produk", to: "/products", icon: Package },
  { label: "Inventori", to: "/inventory", icon: Warehouse },
  { label: "Pembelian", to: "/purchases", icon: ClipboardList },
  { label: "Penjualan", to: "/sales", icon: ShoppingCart },
  { label: "Laporan", to: "/reports", icon: BarChart3 },
  { label: "SMART Ranking", to: "/smart-ranking", icon: Sparkles },
  { label: "Ruang 3D", to: "/warehouse", icon: Map },
  { label: "Riwayat", to: "/history", icon: History },
];
const pageTitle = computed(() => (route.meta.title as string | undefined) ?? "Dashboard");
const appVersion = `v${packageInfo.version}`;

onMounted(async () => {
  try {
    const update = await check();
    if (!update) return;
    const confirmed = window.confirm(`Versi SIAMBRUK ${update.version} tersedia. Instal sekarang?`);
    if (!confirmed) {
      await update.close();
      return;
    }
    await update.downloadAndInstall();
    await relaunch();
  } catch {
    // Update checks are best-effort; the application remains usable offline.
  }
});

let tableObserver: MutationObserver | undefined;
function fillTableSlots() {
  document.querySelectorAll<HTMLTableSectionElement>(".table-panel table tbody").forEach((tbody) => {
    const placeholders = tbody.querySelectorAll(".table-placeholder-row");
    const actualRows = tbody.querySelectorAll(":scope > tr:not(.table-placeholder-row)");
    const missing = Math.max(0, 8 - actualRows.length);
    if (placeholders.length === missing) return;
    placeholders.forEach((row) => row.remove());
    const columnCount = tbody.closest("table")?.querySelectorAll("thead th").length ?? 1;
    for (let index = 0; index < missing; index += 1) {
      const row = document.createElement("tr");
      row.className = "table-placeholder-row";
      row.setAttribute("aria-hidden", "true");
      const cell = document.createElement("td");
      cell.colSpan = columnCount;
      row.append(cell);
      tbody.append(row);
    }
  });
}

onMounted(() => {
  nextTick(fillTableSlots);
  tableObserver = new MutationObserver(fillTableSlots);
  tableObserver.observe(document.body, { childList: true, subtree: true });
});
onUnmounted(() => tableObserver?.disconnect());
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand-lockup">
        <div class="brand-mark"><Store :size="20" stroke-width="2.2" /></div>
        <div><p class="brand-name">SIAMBRUK</p><p class="brand-subtitle">Sistem Informasi Alfa Mabruk</p></div>
      </div>
      <div class="sidebar-section-label">Operasional</div>
      <nav class="navigation" aria-label="Navigasi utama">
        <RouterLink v-for="item in navigation" :key="item.to" :to="item.to" class="nav-item" active-class="nav-item-active" exact-active-class="nav-item-active">
          <component :is="item.icon" :size="18" stroke-width="1.9" /><span>{{ item.label }}</span>
          <ChevronRight v-if="route.path === item.to" class="nav-current" :size="15" />
        </RouterLink>
      </nav>
      <div class="sidebar-footer">
        <RouterLink to="/settings" class="nav-item" active-class="nav-item-active"><Settings :size="18" stroke-width="1.9" /><span>Pengaturan</span></RouterLink>
        <div class="workspace-note"><Boxes :size="17" /><div><strong>Data lokal</strong><span>Siap dikonfigurasi</span></div></div>
      </div>
    </aside>
    <main class="main-content">
      <header class="topbar"><div><p class="eyebrow">Ruang kerja inventori</p><h1>{{ pageTitle }}</h1></div><div class="topbar-meta"><span class="status-dot"></span><span>Mode lokal</span><span class="topbar-divider" aria-hidden="true"></span><span class="topbar-version">{{ appVersion }}</span></div></header>
      <div class="page-content"><RouterView /></div>
    </main>
  </div>
</template>
