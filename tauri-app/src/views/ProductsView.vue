<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { Package, Plus, RefreshCw, X } from "lucide-vue-next";
import PaginationControls from "../components/PaginationControls.vue";

interface Product { id: number; name: string; category_name: string | null; description: string | null; variant_count: number; is_active: boolean }
interface Category { id: number; name: string; description: string | null; product_count: number }
interface Variant { id: number; product_id: number; product_name: string; name: string; sku: string; purchase_price: number; selling_price: number; current_stock: number; minimum_stock: number }

const products = ref<Product[]>([]);
const categories = ref<Category[]>([]);
const variants = ref<Variant[]>([]);
const page = ref(1);
const pageSize = 8;
const selectedProductId = ref<number | null>(null);
const name = ref("");
const categoryId = ref<number | null>(null);
const description = ref("");
const categoryName = ref("");
const editingId = ref<number | null>(null);
const editingVariantId = ref<number | null>(null);
const variantName = ref("");
const variantSku = ref("");
const variantPurchasePrice = ref(0);
const variantSellingPrice = ref(0);
const variantMinimumStock = ref(0);
const adjustmentVariantId = ref<number | null>(null);
const adjustmentQuantity = ref(0);
const adjustmentReason = ref("");
const loading = ref(true);
const saving = ref(false);
const error = ref("");
const message = ref("");

const pagedProducts = computed(() => products.value.slice((page.value - 1) * pageSize, page.value * pageSize));
const selectedProduct = computed(() => products.value.find((product) => product.id === selectedProductId.value) ?? null);
const productVariants = computed(() => variants.value.filter((variant) => variant.product_id === selectedProductId.value));
const money = (value: number) => new Intl.NumberFormat("id-ID", { style: "currency", currency: "IDR", maximumFractionDigits: 0 }).format(value);

async function load() {
  loading.value = true;
  error.value = "";
  try {
    [products.value, categories.value, variants.value] = await Promise.all([
      invoke<Product[]>("list_products"),
      invoke<Category[]>("list_categories"),
      invoke<Variant[]>("list_variants", { productId: null }),
    ]);
    page.value = 1;
  } catch (e) { error.value = String(e); }
  finally { loading.value = false; }
}

function openProduct(product: Product) {
  selectedProductId.value = product.id;
  editingId.value = product.id;
  name.value = product.name;
  categoryId.value = categories.value.find((category) => category.name === product.category_name)?.id ?? null;
  description.value = product.description ?? "";
  cancelVariant();
}

function closeProduct() {
  selectedProductId.value = null;
  cancelProduct();
}

function cancelProduct() {
  editingId.value = null;
  name.value = "";
  categoryId.value = null;
  description.value = "";
  cancelVariant();
}

async function saveProduct() {
  if (!name.value.trim()) { error.value = "Nama produk wajib diisi."; return; }
  saving.value = true; error.value = ""; message.value = "";
  try {
    const input = { name: name.value, category_id: categoryId.value, description: description.value || null };
    if (editingId.value) await invoke("update_product", { input: { ...input, id: editingId.value } });
    else await invoke("create_product", { input });
    await load();
    message.value = "Produk berhasil disimpan.";
  } catch (e) { error.value = String(e); }
  finally { saving.value = false; }
}

async function addCategory() {
  if (!categoryName.value.trim()) { error.value = "Nama kategori wajib diisi."; return; }
  try {
    const category = await invoke<Category>("create_category", { input: { name: categoryName.value, description: null } });
    categories.value.push(category);
    categoryId.value = category.id;
    categoryName.value = "";
    message.value = "Kategori berhasil ditambahkan.";
  } catch (e) { error.value = String(e); }
}

function editVariant(variant: Variant) {
  editingVariantId.value = variant.id;
  variantName.value = variant.name;
  variantSku.value = variant.sku;
  variantPurchasePrice.value = variant.purchase_price;
  variantSellingPrice.value = variant.selling_price;
  variantMinimumStock.value = variant.minimum_stock;
}

function chooseStockAdjustment(variant: Variant) {
  adjustmentVariantId.value = variant.id;
  adjustmentQuantity.value = 0;
  adjustmentReason.value = "";
}

async function adjustVariantStock() {
  if (!adjustmentVariantId.value || adjustmentQuantity.value === 0 || !adjustmentReason.value.trim()) {
    error.value = "Pilih varian, isi perubahan stok, dan alasan koreksi.";
    return;
  }
  try {
    await invoke("adjust_stock", { input: { variant_id: adjustmentVariantId.value, quantity: adjustmentQuantity.value, reason: adjustmentReason.value } });
    adjustmentQuantity.value = 0;
    adjustmentReason.value = "";
    await load();
    message.value = "Stok varian berhasil dikoreksi.";
  } catch (e) { error.value = String(e); }
}

function cancelVariant() {
  editingVariantId.value = null;
  variantName.value = "";
  variantSku.value = "";
  variantPurchasePrice.value = 0;
  variantSellingPrice.value = 0;
  variantMinimumStock.value = 0;
}

async function saveVariant() {
  if (!selectedProductId.value || !variantName.value.trim()) { error.value = "Nama varian wajib diisi."; return; }
  try {
    const sku = variantSku.value.trim() || "SKU-" + Date.now().toString(36).toUpperCase();
    const input = { sku, name: variantName.value, purchase_price: variantPurchasePrice.value, selling_price: variantSellingPrice.value, minimum_stock: variantMinimumStock.value };
    if (editingVariantId.value) await invoke("update_variant", { input: { ...input, id: editingVariantId.value } });
    else await invoke("create_variant", { input: { ...input, product_id: selectedProductId.value } });
    cancelVariant();
    await load();
    message.value = "Varian berhasil disimpan dengan SKU " + sku + ".";
  } catch (e) { error.value = String(e); }
}

async function remove(product: Product) {
  if (!window.confirm("Hapus produk " + product.name + "? Riwayat tetap disimpan, tetapi stok aktif harus dikosongkan.")) return;
  try { await invoke("delete_product", { productId: product.id }); closeProduct(); await load(); }
  catch (e) { error.value = String(e); }
}

onMounted(load);
function closeOnEscape(event: KeyboardEvent) { if (event.key === "Escape" && selectedProductId.value !== null) closeProduct(); }
watch(selectedProductId, (id) => document.body.classList.toggle("modal-open", id !== null), { immediate: true });
onMounted(() => document.addEventListener("keydown", closeOnEscape));
onUnmounted(() => { document.removeEventListener("keydown", closeOnEscape); document.body.classList.remove("modal-open"); });
</script>

<template>
  <section class="module-page">
    <div class="section-intro"><div><p class="eyebrow">Katalog</p><h2>Produk</h2><p class="intro-copy">Kelola produk, kategori, varian, harga beli, dan harga jual dari satu ruang kerja.</p></div><button class="secondary-button" type="button" @click="load"><RefreshCw :size="15" /> Segarkan</button></div>
    <p v-if="error" class="form-error">{{ error }}</p><p v-if="message" class="form-success">{{ message }}</p>
    <div v-if="selectedProduct" class="product-modal-backdrop" role="presentation" @click.self="closeProduct">
      <section class="panel form-panel product-modal" role="dialog" aria-modal="true" :aria-label="`Detail produk ${selectedProduct.name}`">
        <div class="panel-heading"><div><p class="eyebrow">Detail produk</p><h3>{{ selectedProduct.name }}</h3></div><button class="icon-button" type="button" @click="closeProduct"><X :size="17" /></button></div>
        <label>Nama produk<input v-model="name" /></label><label>Kategori<select v-model="categoryId"><option :value="null">Tanpa kategori</option><option v-for="category in categories" :key="category.id" :value="category.id">{{ category.name }}</option></select></label>
        <div class="form-row"><input v-model="categoryName" placeholder="Nama kategori baru" /><button class="secondary-button" type="button" @click="addCategory">Tambah kategori</button></div>
        <label>Deskripsi<textarea v-model="description" rows="3"></textarea></label><div class="action-row"><button class="primary-button" :disabled="saving" type="button" @click="saveProduct">Simpan produk</button><button class="secondary-button" type="button" @click="remove(selectedProduct)">Hapus</button></div>
        <div class="panel-heading variant-heading"><div><p class="eyebrow">Kemasan dan harga</p><h3>Varian produk</h3></div></div>
        <div v-if="!productVariants.length" class="table-state"><Package :size="20" /><span>Belum ada varian.</span></div>
        <div v-for="variant in productVariants" :key="variant.id" class="variant-edit-row"><div><strong>{{ variant.name }}</strong><small>{{ variant.sku }} · stok {{ variant.current_stock }}</small></div><div><span>{{ money(variant.purchase_price) }}</span><span>{{ money(variant.selling_price) }}</span><button class="secondary-button" type="button" @click="editVariant(variant)">Edit</button><button class="secondary-button" type="button" @click="chooseStockAdjustment(variant)">Koreksi stok</button></div></div>
        <div v-if="adjustmentVariantId" class="variant-editor"><p class="eyebrow">Koreksi stok</p><p class="field-hint">Gunakan angka positif untuk menambah stok dan angka negatif untuk mengurangi stok. Nilai inventori ikut dihitung ulang secara aman.</p><div class="form-row"><label>Perubahan stok<input v-model.number="adjustmentQuantity" type="number" step="1" /></label><label>Alasan<input v-model="adjustmentReason" placeholder="Contoh: hasil stok opname" /></label></div><button class="primary-button" type="button" @click="adjustVariantStock">Simpan koreksi</button></div>
        <div class="variant-editor"><p class="eyebrow">{{ editingVariantId ? "Edit varian" : "Tambah varian" }}</p><div class="form-row"><label>Nama varian<input v-model="variantName" placeholder="250gr" /></label><label>SKU <span class="optional">opsional</span><input v-model="variantSku" placeholder="Otomatis jika kosong" /></label></div><div class="form-row"><label>Harga beli default<input v-model.number="variantPurchasePrice" type="number" min="0" /></label><label>Harga jual<input v-model.number="variantSellingPrice" type="number" min="0" /></label></div><label>Batas stok minimum<input v-model.number="variantMinimumStock" type="number" min="0" /></label><div class="action-row"><button class="primary-button" type="button" @click="saveVariant">Simpan varian</button><button v-if="editingVariantId" class="secondary-button" type="button" @click="cancelVariant">Batal</button></div></div>
      </section>
    </div>
    <div class="content-columns">
      <form class="panel form-panel" @submit.prevent="saveProduct"><div class="panel-heading"><div><p class="eyebrow">Data baru</p><h3>Tambah produk</h3></div><Plus :size="18" class="panel-icon" /></div><label>Nama produk<input v-model="name" placeholder="Contoh: Keripik Balado" /></label><label>Kategori<select v-model="categoryId"><option :value="null">Tanpa kategori</option><option v-for="category in categories" :key="category.id" :value="category.id">{{ category.name }}</option></select></label><div class="form-row"><input v-model="categoryName" placeholder="Nama kategori baru" /><button class="secondary-button" type="button" @click="addCategory">Tambah kategori</button></div><label>Deskripsi<textarea v-model="description" rows="4"></textarea></label><button class="primary-button" :disabled="saving" type="submit">Simpan produk</button></form>
      <section class="panel table-panel paginated-panel"><div class="panel-heading"><div><p class="eyebrow">Daftar aktif</p><h3>Produk</h3></div><span class="neutral-chip">{{ products.length }} produk</span></div><div v-if="loading" class="table-state">Memuat data...</div><div v-else-if="!products.length" class="table-state"><Package :size="22" /><strong>Belum ada produk</strong><span>Tambahkan produk untuk mulai membangun katalog.</span></div><template v-else><table><thead><tr><th>Nama produk</th><th>Kategori</th><th>Varian</th><th>Aksi</th></tr></thead><tbody><tr v-for="product in pagedProducts" :key="product.id" @click="openProduct(product)"><td class="cell-strong">{{ product.name }}</td><td>{{ product.category_name || "—" }}</td><td>{{ product.variant_count }}</td><td class="row-actions"><button type="button" @click.stop="openProduct(product)">Detail</button><button type="button" @click.stop="remove(product)">Hapus</button></td></tr></tbody></table><PaginationControls v-model:page="page" :total="products.length" :page-size="pageSize" /></template></section>
    </div>
  </section>
</template>
