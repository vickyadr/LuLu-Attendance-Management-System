<script setup>
// One pager for every server-side list: the parent owns the page ref, this only
// reports what the reader picked.
const props = defineProps({
    page: { type: Number, default: 0 },
    pageSize: { type: Number, default: 50 },
    total: { type: Number, default: 0 },
})
const emit = defineEmits(['update:page'])

const pages = computed(() => Math.max(1, Math.ceil(props.total / props.pageSize)))
const from = computed(() => (props.total ? props.page * props.pageSize + 1 : 0))
const to = computed(() => Math.min(props.total, (props.page + 1) * props.pageSize))
</script>

<template>
    <div v-if="total > pageSize"
        class="px-3 py-2 border-t border-emerald-100 bg-white flex items-center justify-between gap-2">
        <span class="mono text-[11px] text-emerald-700">
            {{ $t('common.pageRange', { from, to, total }) }}
        </span>
        <div class="flex items-center gap-1">
            <button type="button" :disabled="page === 0" :aria-label="$t('common.prevPage')"
                class="h-7 px-2.5 rounded-lg border border-emerald-200 bg-white text-emerald-800 mono text-[11px] font-bold disabled:opacity-40 hover:bg-emerald-50"
                @click="emit('update:page', page - 1)">{{ $t('common.prev') }}</button>
            <span class="mono text-[11px] text-emerald-700 px-1">{{ page + 1 }} / {{ pages }}</span>
            <button type="button" :disabled="page >= pages - 1" :aria-label="$t('common.nextPage')"
                class="h-7 px-2.5 rounded-lg border border-emerald-200 bg-white text-emerald-800 mono text-[11px] font-bold disabled:opacity-40 hover:bg-emerald-50"
                @click="emit('update:page', page + 1)">{{ $t('common.next') }}</button>
        </div>
    </div>
</template>
