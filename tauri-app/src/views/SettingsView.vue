<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, ref } from "vue";
import { Database, Scale, ShieldCheck } from "lucide-vue-next";
interface Status { ready: boolean; schema_version: number; path: string }
const status = ref<Status | null>(null); const error = ref("");
onMounted(async () => { try { status.value = await invoke<Status>("database_status"); } catch (e) { error.value = String(e); } });
</script>

<template><section class="module-page"><div class="section-intro"><div><p class="eyebrow">Konfigurasi</p><h2>Pengaturan sistem</h2><p class="intro-copy">Informasi lingkungan lokal dan metodologi yang digunakan SIAMBRUK.</p></div></div><p v-if="error" class="form-error">{{ error }}</p><div class="settings-grid"><section class="panel settings-card"><Database :size="20" class="panel-icon" /><p class="eyebrow">Database lokal</p><h3>{{ status?.ready ? "Siap digunakan" : "Memuat status" }}</h3><p>{{ status ? `Schema v${status.schema_version}` : "Status database akan tampil setelah aplikasi desktop aktif." }}</p><code v-if="status">{{ status.path }}</code></section><section class="panel settings-card"><Scale :size="20" class="panel-icon" /><p class="eyebrow">SMART methodology</p><h3>Bobot evaluasi awal</h3><div class="settings-lines"><span>Keuntungan <strong>30%</strong></span><span>Produk terjual <strong>40%</strong></span><span>Stok <strong>15%</strong></span><span>Harga pembelian <strong>15%</strong></span></div></section><section class="panel settings-card"><ShieldCheck :size="20" class="panel-icon" /><p class="eyebrow">Keamanan operasi</p><h3>Transaksi tervalidasi</h3><p>Perubahan stok, pembelian, dan penjualan menggunakan transaksi SQLite dan tidak dijalankan dari animasi visual semata.</p></section></div></section></template>
