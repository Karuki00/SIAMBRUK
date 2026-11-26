<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, ref } from "vue";
import { BarChart3, Download, RefreshCw } from "lucide-vue-next";

interface Summary { active_products: number; variant_count: number; total_stock: number; sales_today: number; revenue_today: number; profit_today: number; low_stock_count: number }
interface Variant { id: number; product_id: number; product_name: string; name: string; sku: string; current_stock: number; purchase_price: number; selling_price: number }
interface ReportPoint { date: string; units_sold: number; revenue: number; profit: number }
interface VariantReport { variant_id: number; product_name: string; variant_name: string; current_stock: number; units_sold: number; revenue: number; profit: number; points: ReportPoint[] }

const today = new Date().toISOString().slice(0, 10);
const periodStart = ref(today.slice(0, 8) + "01");
const periodEnd = ref(today);
const summary = ref<Summary | null>(null);
const variants = ref<Variant[]>([]);
const selectedProductId = ref<number | null>(null);
const selectedVariantId = ref<number | null>(null);
const variantReport = ref<VariantReport | null>(null);
const error = ref("");
const loading = ref(true);
const statsLoading = ref(false);
const money = (value: number) => new Intl.NumberFormat("id-ID", { style: "currency", currency: "IDR", maximumFractionDigits: 0 }).format(value);
const productOptions = computed(() => Array.from(new Map(variants.value.map((variant) => [variant.product_id, { id: variant.product_id, name: variant.product_name }])).values()));
const selectedProductVariants = computed(() => variants.value.filter((variant) => variant.product_id === selectedProductId.value));
const chartWidth = 720;
const chartHeight = 270;
const chartPadding = { top: 20, right: 24, bottom: 36, left: 54 };
const chartPoints = computed(() => {
  const points = variantReport.value?.points ?? [];
  const max = Math.max(...points.map((point) => point.revenue), 1);
  const innerWidth = chartWidth - chartPadding.left - chartPadding.right;
  const innerHeight = chartHeight - chartPadding.top - chartPadding.bottom;
  return points.map((point, index) => ({ ...point, x: points.length === 1 ? chartPadding.left + innerWidth / 2 : chartPadding.left + (index / (points.length - 1)) * innerWidth, y: chartPadding.top + innerHeight - (point.revenue / max) * innerHeight }));
});
const linePath = computed(() => chartPoints.value.map((point, index) => (index ? "L " : "M ") + point.x + " " + point.y).join(" "));
const chartMax = computed(() => Math.max(...(variantReport.value?.points ?? []).map((point) => point.revenue), 1));

async function loadStats() {
  if (!selectedVariantId.value) { variantReport.value = null; return; }
  statsLoading.value = true;
  try {
    variantReport.value = await invoke<VariantReport>("variant_report_stats", { request: { variant_id: selectedVariantId.value, period_start: periodStart.value, period_end: periodEnd.value } });
  } catch (e) { error.value = String(e); }
  finally { statsLoading.value = false; }
}

function chooseProduct() {
  selectedVariantId.value = selectedProductVariants.value[0]?.id ?? null;
  void loadStats();
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    [summary.value, variants.value] = await Promise.all([
      invoke<Summary>("report_summary_range", { request: { period_start: periodStart.value, period_end: periodEnd.value } }),
      invoke<Variant[]>("list_variants", { productId: null }),
    ]);
    if (!selectedProductId.value || !productOptions.value.some((product) => product.id === selectedProductId.value)) selectedProductId.value = productOptions.value[0]?.id ?? null;
    if (!selectedVariantId.value || !selectedProductVariants.value.some((variant) => variant.id === selectedVariantId.value)) selectedVariantId.value = selectedProductVariants.value[0]?.id ?? null;
    await loadStats();
  } catch (e) { error.value = String(e); }
  finally { loading.value = false; }
}

function exportCsv() {
  if (!summary.value) return;
  const rows = [["Metrik", "Nilai"], ["Periode", periodStart.value + " - " + periodEnd.value], ["Produk aktif", summary.value.active_products], ["Total varian", summary.value.variant_count], ["Total stok", summary.value.total_stock], ["Penjualan", summary.value.sales_today], ["Pendapatan", summary.value.revenue_today], ["Laba", summary.value.profit_today], ["Stok menipis", summary.value.low_stock_count]];
  const csv = rows.map((row) => row.map((value) => "\"" + String(value).replace(/"/g, "\"\"") + "\"").join(",")).join("\r\n");
  const url = URL.createObjectURL(new Blob([csv], { type: "text/csv;charset=utf-8" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = "siambruk-report-" + periodStart.value + "-" + periodEnd.value + ".csv";
  link.click();
  URL.revokeObjectURL(url);
}

onMounted(load);
</script>

<template>
  <section class="module-page">
    <div class="section-intro"><div><p class="eyebrow">Analisis</p><h2>Laporan ringkas</h2><p class="intro-copy">Pilih periode untuk melihat ringkasan dan tren penjualan berdasarkan produk serta varian.</p></div><div class="action-row"><button class="secondary-button" type="button" @click="load"><RefreshCw :size="15" /> Segarkan</button><button class="secondary-button" :disabled="!summary" type="button" @click="exportCsv"><Download :size="15" /> CSV</button></div></div>
    <section class="panel evaluation-controls"><div><p class="eyebrow">Periode laporan</p><div class="form-row"><label>Mulai<input v-model="periodStart" type="date" /></label><label>Selesai<input v-model="periodEnd" type="date" /></label></div></div><button class="primary-button" type="button" @click="load">Hitung laporan</button></section>
    <p v-if="error" class="form-error">{{ error }}</p>
    <section class="metric-grid report-metrics"><article class="metric-card"><span class="metric-label">Produk aktif</span><strong class="metric-value">{{ summary?.active_products ?? "—" }}</strong></article><article class="metric-card"><span class="metric-label">Total stok</span><strong class="metric-value">{{ summary?.total_stock ?? "—" }}</strong></article><article class="metric-card"><span class="metric-label">Penjualan periode</span><strong class="metric-value">{{ summary?.sales_today ?? "—" }}</strong></article><article class="metric-card"><span class="metric-label">Pendapatan periode</span><strong class="metric-value report-money">{{ summary ? money(summary.revenue_today) : "—" }}</strong></article></section>
    <section class="panel report-panel product-report-panel"><div class="panel-heading"><div><p class="eyebrow">Statistik produk</p><h3>Tren penjualan</h3></div><BarChart3 :size="18" class="panel-icon" /></div><div class="report-filters"><label>Produk<select v-model="selectedProductId" @change="chooseProduct"><option :value="null">Pilih produk</option><option v-for="product in productOptions" :key="product.id" :value="product.id">{{ product.name }}</option></select></label><label>Varian<select v-model="selectedVariantId" @change="loadStats"><option :value="null">Pilih varian</option><option v-for="variant in selectedProductVariants" :key="variant.id" :value="variant.id">{{ variant.name }} · {{ variant.sku }}</option></select></label></div><div v-if="statsLoading" class="table-state report-chart-state">Memuat statistik...</div><div v-else-if="!variantReport" class="table-state report-chart-state"><BarChart3 :size="22" /><strong>Belum ada varian dipilih</strong><span>Pilih produk dan varian untuk melihat tren penjualan.</span></div><div v-else class="product-report-layout"><div class="line-chart-wrap"><div class="chart-heading"><div><strong>{{ variantReport.product_name }} / {{ variantReport.variant_name }}</strong><span>Pendapatan harian</span></div><strong>{{ money(variantReport.revenue) }}</strong></div><div v-if="!chartPoints.length" class="chart-empty">Belum ada penjualan pada periode ini.</div><svg v-else class="line-chart" :viewBox="'0 0 ' + chartWidth + ' ' + chartHeight" role="img" :aria-label="'Grafik pendapatan ' + variantReport.variant_name"><line :x1="chartPadding.left" :y1="chartHeight - chartPadding.bottom" :x2="chartWidth - chartPadding.right" :y2="chartHeight - chartPadding.bottom" class="chart-axis" /><line :x1="chartPadding.left" :y1="chartPadding.top" :x2="chartPadding.left" :y2="chartHeight - chartPadding.bottom" class="chart-axis" /><line :x1="chartPadding.left" :y1="chartPadding.top" :x2="chartWidth - chartPadding.right" :y2="chartPadding.top" class="chart-grid" /><path :d="linePath" class="chart-line" /><g v-for="point in chartPoints" :key="point.date"><circle :cx="point.x" :cy="point.y" r="4" class="chart-point" /><text :x="point.x" :y="chartHeight - 12" text-anchor="middle" class="chart-label">{{ point.date.slice(5) }}</text></g><text :x="chartPadding.left - 8" :y="chartPadding.top + 4" text-anchor="end" class="chart-label">{{ money(chartMax) }}</text><text :x="chartPadding.left - 8" :y="chartHeight - chartPadding.bottom + 4" text-anchor="end" class="chart-label">Rp0</text></svg></div><aside class="report-stat-list"><p class="eyebrow">Ringkasan varian</p><div><span>Unit terjual</span><strong>{{ variantReport.units_sold }}</strong></div><div><span>Pendapatan</span><strong>{{ money(variantReport.revenue) }}</strong></div><div><span>Laba tercatat</span><strong>{{ money(variantReport.profit) }}</strong></div><div><span>Stok saat ini</span><strong>{{ variantReport.current_stock }}</strong></div></aside></div></section>
    <section class="panel report-panel"><div class="panel-heading"><div><p class="eyebrow">Kesehatan operasional</p><h3>Ringkasan periode</h3></div><BarChart3 :size="18" class="panel-icon" /></div><div v-if="loading" class="table-state">Menghitung ringkasan...</div><div v-else class="report-list"><div><span>Laba tercatat</span><strong>{{ money(summary?.profit_today ?? 0) }}</strong></div><div><span>Varian stok menipis</span><strong>{{ summary?.low_stock_count ?? 0 }}</strong></div><div><span>Total varian</span><strong>{{ summary?.variant_count ?? 0 }}</strong></div></div></section>
  </section>
</template>
