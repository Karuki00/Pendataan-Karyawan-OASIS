<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  modelValue: number;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void;
}>();

// Dynamic year generator from 2024 up to currentYear + 5
const availableYears = computed(() => {
  const currentYear = new Date().getFullYear();
  const startYear = 2024;
  const maxYear = Math.max(currentYear + 5, 2030);
  const years: number[] = [];
  for (let y = startYear; y <= maxYear; y++) {
    years.push(y);
  }
  return years;
});
</script>

<template>
  <div class="flex items-center gap-2.5">
    <label class="text-xs font-bold text-slate-600">Pilih Tahun:</label>
    <select
      :value="modelValue"
      @change="emit('update:modelValue', Number(($event.target as HTMLSelectElement).value))"
      class="text-xs font-bold px-3 py-1.5 rounded-xl border border-slate-300 bg-white text-slate-800 focus:outline-none focus:ring-2 focus:ring-emerald-500 shadow-2xs cursor-pointer"
    >
      <option v-for="y in availableYears" :key="y" :value="y">{{ y }}</option>
    </select>
  </div>
</template>