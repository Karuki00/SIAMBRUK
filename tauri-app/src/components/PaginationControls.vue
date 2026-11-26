<script setup lang="ts">
import { computed, watch } from "vue";

const props = withDefaults(defineProps<{ page: number; total: number; pageSize?: number }>(), { pageSize: 8 });
const emit = defineEmits<{ "update:page": [page: number] }>();
const totalPages = computed(() => Math.max(1, Math.ceil(props.total / props.pageSize)));
function setPage(page: number) { emit("update:page", Math.min(Math.max(page, 1), totalPages.value)); }
watch([() => props.total, totalPages], () => {
  if (props.page > totalPages.value) emit("update:page", totalPages.value);
}, { immediate: true });
</script>

<template>
  <div v-if="total > 0" class="pagination" aria-label="Paginasi daftar">
    <span>Halaman {{ page }} dari {{ totalPages }}</span>
    <div class="pagination-actions"><button type="button" :disabled="page <= 1" aria-label="Halaman sebelumnya" @click="setPage(page - 1)">Sebelumnya</button><button type="button" :disabled="page >= totalPages" aria-label="Halaman berikutnya" @click="setPage(page + 1)">Berikutnya</button></div>
  </div>
</template>
