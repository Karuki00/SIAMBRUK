<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, ref, watch } from "vue";
import { ClipboardList, Plus, ShoppingCart, Trash2 } from "lucide-vue-next";
import { useRoute } from "vue-router";

interface Variant { id: number; product_id: number; product_name: string; name: string; sku: string; current_stock: number; purchase_price: number; selling_price: number }
interface Transaction { id: number; total_amount: number; total_profit: number }
interface TransactionHistory { id: number; reference: string | null; date: string; total_amount: number; total_profit: number }
interface PurchaseLine { variantId: number | null; unitsProduced: number }

const route = useRoute();
const isPurchase = computed(() => route.path === "/purchases");
const variants = ref<Variant[]>([]);
const variantId = ref<number | null>(null);
const quantity = ref(1);
const unitValue = ref(0);
const balPrice = ref(0);
const balCount = ref(1);
const purchaseLines = ref<PurchaseLine[]>([{ variantId: null, unitsProduced: 1 }]);
const date = ref(new Date().toISOString().slice(0, 10));
const reference = ref("");
const notes = ref("");
const loading = ref(false);
const error = ref("");
const message = ref("");
const lastTransaction = ref<Transaction | null>(null);
const transactionHistory = ref<TransactionHistory[]>([]);
const totalUnitsPerBal = computed(() => purchaseLines.value.reduce((sum, line) => sum + Math.max(0, Number(line.unitsProduced) || 0), 0));
const purchaseUnitCost = computed(() => totalUnitsPerBal.value > 0 ? Math.round(balPrice.value / totalUnitsPerBal.value) : 0);
const totalPurchaseUnits = computed(() => balCount.value * totalUnitsPerBal.value);
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
  } catch (e) { error.value = String(e); }
}

function resetForm() {
  variantId.value = null;
  quantity.value = 1;
  unitValue.value = 0;
  balPrice.value = 0;
  balCount.value = 1;
  purchaseLines.value = [{ variantId: null, unitsProduced: 1 }];
  date.value = new Date().toISOString().slice(0, 10);
  reference.value = "";
  notes.value = "";
  loading.value = false;
  error.value = "";
  message.value = "";
  lastTransaction.value = null;
  transactionHistory.value = [];
}

function chooseSaleVariant() {
  const variant = variants.value.find((item) => item.id === variantId.value);
  if (variant) unitValue.value = variant.selling_price;
}

function addPurchaseLine() { purchaseLines.value.push({ variantId: null, unitsProduced: 1 }); }
function removePurchaseLine(index: number) { if (purchaseLines.value.length > 1) purchaseLines.value.splice(index, 1); }

async function submit() {
  error.value = "";
  message.value = "";
  if (!date.value) { error.value = "Tanggal wajib diisi."; return; }
  if (isPurchase.value) {
    const validLines = purchaseLines.value.filter((line) => line.variantId && line.unitsProduced > 0);
    if (balCount.value <= 0 || balPrice.value <= 0 || !validLines.length || validLines.length !== purchaseLines.value.length || totalUnitsPerBal.value <= 0) {
      error.value = "Isi jumlah bal, harga bal, dan minimal satu varian dengan unit hasil yang valid.";
      return;
    }
  } else if (!variantId.value || quantity.value <= 0 || unitValue.value < 0) {
    error.value = "Varian, jumlah unit, dan harga jual yang valid wajib diisi.";
    return;
  }
  loading.value = true;
  try {
    const input = isPurchase.value
      ? { reference: reference.value || null, purchased_at: date.value, notes: notes.value || null, items: purchaseLines.value.map((line) => ({ variant_id: line.variantId, quantity: balCount.value * line.unitsProduced, unit_cost: purchaseUnitCost.value })) }
      : { reference: reference.value || null, sold_at: date.value, notes: notes.value || null, items: [{ variant_id: variantId.value, quantity: quantity.value, unit_price: unitValue.value }] };
    lastTransaction.value = await invoke<Transaction>(isPurchase.value ? "create_purchase" : "create_sale", { input });
    message.value = isPurchase.value ? "Pembelian berhasil dicatat." : "Penjualan berhasil dicatat.";
    quantity.value = 1;
    unitValue.value = 0;
    balPrice.value = 0;
    balCount.value = 1;
    purchaseLines.value = [{ variantId: null, unitsProduced: 1 }];
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
    <div class="section-intro"><div><p class="eyebrow">Transaksi</p><h2>{{ isPurchase ? "Pembelian dan stok masuk" : "Penjualan" }}</h2><p class="intro-copy">{{ isPurchase ? "Satu bal dapat dikemas menjadi beberapa varian dengan biaya unit yang dibagi dari total hasil kemasan." : "Catat penjualan dengan harga jual varian yang dapat disesuaikan." }}</p></div></div>
    <p v-if="error" class="form-error">{{ error }}</p><p v-if="message" class="form-success">{{ message }}</p>
    <div class="content-columns">
      <form class="panel form-panel" @submit.prevent="submit">
        <div class="panel-heading"><div><p class="eyebrow">Transaksi baru</p><h3>{{ isPurchase ? "Stok masuk dari bal" : "Penjualan" }}</h3></div><component :is="isPurchase ? ClipboardList : ShoppingCart" :size="18" class="panel-icon" /></div>
        <div v-if="isPurchase" class="form-row"><label>Tanggal<input v-model="date" type="date" /></label><label>Jumlah bal<input v-model.number="balCount" type="number" min="1" step="1" /></label></div>
        <label v-if="isPurchase">Harga total per bal<input v-model.number="balPrice" type="number" min="1" step="1" /></label>
        <template v-if="isPurchase"><div class="panel-heading compact-heading"><div><p class="eyebrow">Hasil kemasan</p><h3>Varian dari bal</h3></div><button class="secondary-button" type="button" @click="addPurchaseLine"><Plus :size="15" /> Tambah varian</button></div><div v-for="(line, index) in purchaseLines" :key="index" class="purchase-line"><label>Varian<select v-model="line.variantId"><option :value="null">Pilih varian</option><option v-for="variant in variants" :key="variant.id" :value="variant.id">{{ variant.product_name }} — {{ variant.name }}</option></select></label><label>Unit hasil per bal<input v-model.number="line.unitsProduced" type="number" min="1" step="1" /></label><button v-if="purchaseLines.length > 1" class="icon-button" type="button" @click="removePurchaseLine(index)"><Trash2 :size="16" /></button></div><p class="field-hint">{{ balCount }} bal menghasilkan {{ totalPurchaseUnits }} unit. Biaya dasar per unit: <strong>{{ formatMoney(purchaseUnitCost) }}</strong> · total pembelian: <strong>{{ formatMoney(balCount * balPrice) }}</strong></p></template>
        <template v-else><div class="form-row"><label>Tanggal<input v-model="date" type="date" /></label><label>Jumlah unit<input v-model.number="quantity" type="number" min="1" step="1" /></label></div><label>Varian<select v-model="variantId" @change="chooseSaleVariant"><option :value="null">Pilih varian</option><option v-for="variant in variants" :key="variant.id" :value="variant.id">{{ variant.product_name }} — {{ variant.name }} (stok {{ variant.current_stock }})</option></select></label><label>Harga jual per unit<input v-model.number="unitValue" type="number" min="0" /></label></template>
        <label>Referensi <span class="optional">opsional</span><input v-model="reference" placeholder="Nomor nota atau referensi" /></label><label>Catatan <span class="optional">opsional</span><textarea v-model="notes" rows="3"></textarea></label><button class="primary-button" :disabled="loading" type="submit">Simpan {{ isPurchase ? "pembelian" : "penjualan" }}</button>
      </form>
      <section class="panel transaction-control"><div class="panel-heading"><div><p class="eyebrow">Kontrol transaksi</p><h3>Nilai tercatat</h3></div></div><div v-if="lastTransaction" class="transaction-result"><strong>{{ formatMoney(lastTransaction.total_amount) }}</strong><span>Total transaksi #{{ lastTransaction.id }}</span><span v-if="!isPurchase">Laba tercatat: {{ formatMoney(lastTransaction.total_profit) }}</span></div><div v-else-if="!transactionHistory.length" class="table-state"><component :is="isPurchase ? ClipboardList : ShoppingCart" :size="22" /><strong>Belum ada transaksi</strong><span>Transaksi yang berhasil akan muncul di sini.</span></div><div v-if="transactionHistory.length" class="transaction-history"><div v-for="item in transactionHistory.slice(0, 5)" :key="item.id" class="transaction-history-row"><span><strong>{{ item.reference || ("#" + item.id) }}</strong><small>{{ item.date }}</small></span><span><strong>{{ formatMoney(item.total_amount) }}</strong><small v-if="!isPurchase">Laba {{ formatMoney(item.total_profit) }}</small></span></div></div></section>
    </div>
  </section>
</template>
