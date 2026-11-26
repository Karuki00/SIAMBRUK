<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, ref, watch } from "vue";
import { ClipboardList, ShoppingCart } from "lucide-vue-next";
import { useRoute } from "vue-router";

interface Variant { id: number; product_name: string; name: string; sku: string; current_stock: number; purchase_price: number; selling_price: number }
interface Transaction { id: number; total_amount: number; total_profit: number }
interface TransactionHistory { id: number; reference: string | null; date: string; total_amount: number; total_profit: number }

const route = useRoute();
const isPurchase = computed(() => route.path === "/purchases");
const variants = ref<Variant[]>([]);
const variantId = ref<number | null>(null);
const quantity = ref(1);
const unitValue = ref(0);
const date = ref(new Date().toISOString().slice(0, 10));
const reference = ref("");
const notes = ref("");
const loading = ref(false);
const error = ref("");
const message = ref("");
const lastTransaction = ref<Transaction | null>(null);
const transactionHistory = ref<TransactionHistory[]>([]);

const formatMoney = (value: number) => new Intl.NumberFormat("id-ID", { style: "currency", currency: "IDR", maximumFractionDigits: 0 }).format(value);

async function load() {
  try {
    variants.value = await invoke<Variant[]>("list_variants", { productId: null });
    if (isPurchase.value) {
      const rows = await invoke<Array<{ id: number; reference: string | null; purchased_at: string; total_amount: number }>>("list_purchases_history");
      transactionHistory.value = rows.map((row) => ({ id: row.id, reference: row.reference, date: row.purchased_at, total_amount: row.total_amount, total_profit: 0 }));
    } else {
      const rows = await invoke<Array<{ id: number; reference: string | null; sold_at: string; total_amount: number; total_profit: number }>>("list_sales_history");
      transactionHistory.value = rows.map((row) => ({ id: row.id, reference: row.reference, date: row.sold_at, total_amount: row.total_amount, total_profit: row.total_profit }));
    }
  }
  catch (e) { error.value = String(e); }
}

function resetForm() {
  variantId.value = null;
  quantity.value = 1;
  unitValue.value = 0;
  date.value = new Date().toISOString().slice(0, 10);
  reference.value = "";
  notes.value = "";
  loading.value = false;
  error.value = "";
  message.value = "";
  lastTransaction.value = null;
  transactionHistory.value = [];
}

function chooseVariant() {
  const variant = variants.value.find((item) => item.id === variantId.value);
  if (variant) unitValue.value = isPurchase.value ? variant.purchase_price : variant.selling_price;
}

async function submit() {
  error.value = "";
  message.value = "";
  if (!variantId.value || quantity.value <= 0 || !date.value) { error.value = "Varian, tanggal, dan jumlah yang valid wajib diisi."; return; }
  loading.value = true;
  try {
    const input = isPurchase.value
      ? { reference: reference.value || null, purchased_at: date.value, notes: notes.value || null, items: [{ variant_id: variantId.value, quantity: quantity.value, unit_cost: unitValue.value }] }
      : { reference: reference.value || null, sold_at: date.value, notes: notes.value || null, items: [{ variant_id: variantId.value, quantity: quantity.value, unit_price: unitValue.value }] };
    lastTransaction.value = await invoke<Transaction>(isPurchase.value ? "create_purchase" : "create_sale", { input });
    message.value = `${isPurchase.value ? "Pembelian" : "Penjualan"} berhasil dicatat.`;
    quantity.value = 1;
    reference.value = "";
    notes.value = "";
    await load();
  } catch (e) { error.value = String(e); }
  finally { loading.value = false; }
}

watch(() => route.path, () => { resetForm(); void load(); });
onMounted(load);
</script>

<template>
  <section class="module-page">
    <div class="section-intro"><div><p class="eyebrow">Transaksi</p><h2>{{ isPurchase ? "Pembelian dan stok masuk" : "Penjualan" }}</h2><p class="intro-copy">{{ isPurchase ? "Catat stok masuk melalui transaksi pembelian agar saldo dan riwayat bergerak bersama." : "Catat penjualan dengan validasi stok dan perhitungan laba dari biaya tersimpan." }}</p></div></div>
    <p v-if="error" class="form-error">{{ error }}</p><p v-if="message" class="form-success">{{ message }}</p>
    <div class="content-columns">
      <form class="panel form-panel" @submit.prevent="submit"><div class="panel-heading"><div><p class="eyebrow">Transaksi baru</p><h3>{{ isPurchase ? "Stok masuk" : "Penjualan" }}</h3></div><component :is="isPurchase ? ClipboardList : ShoppingCart" :size="18" class="panel-icon" /></div><label>Varian<select v-model="variantId" @change="chooseVariant"><option :value="null">Pilih varian</option><option v-for="variant in variants" :key="variant.id" :value="variant.id">{{ variant.product_name }} — {{ variant.name }} (stok {{ variant.current_stock }})</option></select></label><div class="form-row"><label>Tanggal<input v-model="date" type="date" /></label><label>Jumlah<input v-model.number="quantity" type="number" min="1" /></label></div><label>{{ isPurchase ? "Harga beli per unit" : "Harga jual per unit" }}<input v-model.number="unitValue" type="number" min="0" /></label><label>Referensi <span class="optional">opsional</span><input v-model="reference" placeholder="Nomor nota atau referensi" /></label><label>Catatan <span class="optional">opsional</span><textarea v-model="notes" rows="3"></textarea></label><button class="primary-button" :disabled="loading" type="submit">{{ loading ? "Menyimpan..." : `Simpan ${isPurchase ? "pembelian" : "penjualan"}` }}</button></form>
      <section class="panel transaction-control"><div class="panel-heading"><div><p class="eyebrow">Kontrol transaksi</p><h3>Nilai tercatat</h3></div></div><div v-if="lastTransaction" class="transaction-result"><strong>{{ formatMoney(lastTransaction.total_amount) }}</strong><span>Total transaksi #{{ lastTransaction.id }}</span><span v-if="!isPurchase">Laba tercatat: {{ formatMoney(lastTransaction.total_profit) }}</span></div><div v-else-if="!transactionHistory.length" class="table-state"><component :is="isPurchase ? ClipboardList : ShoppingCart" :size="22" /><strong>Belum ada transaksi</strong><span>Transaksi yang berhasil akan muncul di sini.</span></div><div v-if="transactionHistory.length" class="transaction-history"><div v-for="item in transactionHistory.slice(0, 5)" :key="item.id" class="transaction-history-row"><span><strong>{{ item.reference || `#${item.id}` }}</strong><small>{{ item.date }}</small></span><span><strong>{{ formatMoney(item.total_amount) }}</strong><small v-if="!isPurchase">Laba {{ formatMoney(item.total_profit) }}</small></span></div></div></section>
    </div>
  </section>
</template>
