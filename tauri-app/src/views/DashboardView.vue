<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, ref } from "vue";
import { ArrowUpRight, PackageCheck, ReceiptText, TriangleAlert } from "lucide-vue-next";

interface DatabaseStatus {
  ready: boolean;
  schema_version: number;
  product_count: number;
  variant_count: number;
  category_count: number;
  path: string;
}
interface ReportSummary { total_stock: number; sales_today: number; low_stock_count: number; revenue_today: number; profit_today: number }

const databaseStatus = ref<DatabaseStatus | null>(null);
const reportSummary = ref<ReportSummary | null>(null);
const databaseError = ref<string | null>(null);

onMounted(async () => {
  try {
    [databaseStatus.value, reportSummary.value] = await Promise.all([
      invoke<DatabaseStatus>("database_status"),
      invoke<ReportSummary>("report_summary"),
    ]);
  } catch {
    databaseError.value = "Desktop database belum terhubung.";
  }
});
</script>

<template>
  <section class="dashboard-intro"><div><p class="eyebrow">Ringkasan bisnis</p><h2>Selamat datang di ruang kerja SIAMBRUK.</h2><p class="intro-copy">Mulai dengan mengatur kategori dan produk agar data inventori dapat digunakan untuk operasional harian.</p></div><div class="intro-tag"><span class="tag-line"></span> Tahap konfigurasi awal</div></section>
  <section class="metric-grid" aria-label="Ringkasan metrik">
    <article class="metric-card"><div class="metric-topline"><span class="metric-icon tone-blue"><PackageCheck :size="18" /></span><span class="metric-label">Produk aktif</span></div><strong class="metric-value">{{ databaseStatus?.product_count ?? "—" }}</strong><span class="metric-note">{{ databaseStatus ? "Terdaftar di database" : "Memuat database" }}</span></article>
    <article class="metric-card"><div class="metric-topline"><span class="metric-icon tone-green"><ReceiptText :size="18" /></span><span class="metric-label">Total stok</span></div><strong class="metric-value">{{ reportSummary?.total_stock ?? "—" }}</strong><span class="metric-note">{{ reportSummary ? "Saldo semua varian" : "Memuat database" }}</span></article>
    <article class="metric-card"><div class="metric-topline"><span class="metric-icon tone-amber"><ArrowUpRight :size="18" /></span><span class="metric-label">Penjualan hari ini</span></div><strong class="metric-value">{{ reportSummary?.sales_today ?? "—" }}</strong><span class="metric-note">{{ reportSummary ? "Unit terjual" : "Memuat database" }}</span></article>
    <article class="metric-card"><div class="metric-topline"><span class="metric-icon tone-red"><TriangleAlert :size="18" /></span><span class="metric-label">Stok menipis</span></div><strong class="metric-value">{{ reportSummary?.low_stock_count ?? "—" }}</strong><span class="metric-note">{{ databaseError ?? (reportSummary ? "Di bawah batas minimum" : "Memuat status") }}</span></article>
  </section>
  <section class="dashboard-grid">
    <article class="panel panel-large"><div class="panel-heading"><div><p class="eyebrow">Aktivitas</p><h3>Pergerakan inventori</h3></div><span class="muted-label">Belum tersedia</span></div><div class="empty-state"><div class="empty-icon"><ReceiptText :size="22" /></div><h4>Belum ada pergerakan stok</h4><p>Catatan pembelian, penjualan, dan penyesuaian stok akan tampil di sini setelah database siap.</p></div></article>
    <article class="panel"><div class="panel-heading"><div><p class="eyebrow">Prioritas</p><h3>SMART Ranking</h3></div><span class="neutral-chip">Draft</span></div><div class="ranking-placeholder"><div class="ranking-line"><span>Keuntungan</span><strong>30%</strong></div><div class="ranking-line"><span>Produk terjual</span><strong>40%</strong></div><div class="ranking-line"><span>Stok</span><strong>15%</strong></div><div class="ranking-line"><span>Harga pembelian</span><strong>15%</strong></div><p>Ranking akan dihitung dari data varian nyata, bukan angka demo.</p></div></article>
  </section>
</template>
