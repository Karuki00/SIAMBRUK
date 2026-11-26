<script setup lang="ts">
interface Variant { id: number; product_name: string; name: string; sku: string; current_stock: number; minimum_stock: number; purchase_price: number; selling_price: number }
defineProps<{ variants: Variant[]; selectedId: number | null }>();
const emit = defineEmits<{ select: [variant: Variant] }>();
</script>

<template>
  <section class="warehouse-accessible-list" aria-label="Toegankelijke lijst van inventori">
    <div class="warehouse-list-heading"><p class="eyebrow">Alternatif keyboard</p><span>Gunakan Tab untuk memilih varian</span></div>
    <div class="warehouse-list-items"><button v-for="variant in variants" :key="variant.id" class="warehouse-list-item" :class="{ selected: selectedId === variant.id }" type="button" @click="emit('select', variant)" @keydown.enter="emit('select', variant)"><span>{{ variant.product_name }} — {{ variant.name }}</span><strong>Stok {{ variant.current_stock }}</strong><small>{{ variant.sku }}</small></button><p v-if="!variants.length" class="warehouse-list-empty">Belum ada varian.</p></div>
  </section>
</template>
