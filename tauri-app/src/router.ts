import { createRouter, createWebHashHistory } from "vue-router";
import DashboardView from "./views/DashboardView.vue";
import ProductsView from "./views/ProductsView.vue";
import InventoryView from "./views/InventoryView.vue";
import SmartRankingView from "./views/SmartRankingView.vue";
import OperationsView from "./views/OperationsView.vue";
import ReportsView from "./views/ReportsView.vue";
import WarehouseView from "./views/WarehouseView.vue";
import HistoryView from "./views/HistoryView.vue";
import SettingsView from "./views/SettingsView.vue";

const routes = [
  { path: "/", name: "dashboard", component: DashboardView, meta: { title: "Dashboard" } },
  { path: "/products", component: ProductsView, meta: { title: "Produk" } },
  { path: "/categories", redirect: "/products" },
  { path: "/inventory", component: InventoryView, meta: { title: "Inventori" } },
  { path: "/purchases", component: OperationsView, meta: { title: "Pembelian" } },
  { path: "/sales", component: OperationsView, meta: { title: "Penjualan" } },
  { path: "/reports", component: ReportsView, meta: { title: "Laporan" } },
  { path: "/smart-ranking", component: SmartRankingView, meta: { title: "SMART Ranking" } },
  { path: "/warehouse", component: WarehouseView, meta: { title: "Ruang 3D" } },
  { path: "/history", component: HistoryView, meta: { title: "Riwayat" } },
  { path: "/settings", component: SettingsView, meta: { title: "Pengaturan" } },
];

export const router = createRouter({ history: createWebHashHistory(), routes });
