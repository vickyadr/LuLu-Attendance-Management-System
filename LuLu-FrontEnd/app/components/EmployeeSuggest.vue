<script setup>
// Employee picker used where a plain select listed raw ids. The menu is
// teleported to <body>: a card with its own stacking context can no longer
// clip or paint over it.
const props = defineProps({
    employees: { type: Array, default: () => [] },
    placeholder: { type: String, default: '' },
    inputClass: { type: String, default: '' },
})
const emit = defineEmits(['pick', 'clear'])

const query = ref('')
const open = ref(false)
const inputEl = ref(null)
const menuStyle = ref({})

function label(e) {
    const name = [e.first_name || e.name, e.last_name].filter(Boolean).join(' ')
    return e.department ? `${name} - ${e.department}` : name
}
function nameOf(e) {
    return [e.first_name || e.name, e.last_name].filter(Boolean).join(" ")
}

const matches = computed(() => {
    const q = query.value.trim().toLowerCase()
    const rows = props.employees || []
    const list = q ? rows.filter((e) => label(e).toLowerCase().includes(q)) : rows
    return list.slice(0, 8)
})

function place() {
    const el = inputEl.value
    if (!el) return
    const rect = el.getBoundingClientRect()
    const width = Math.min(320, Math.max(220, rect.width))
    const height = Math.min(224, matches.value.length * 38 + 8)
    const left = Math.min(Math.max(8, rect.left), window.innerWidth - width - 8)
    let top = rect.bottom + 4
    if (top + height > window.innerHeight - 8 && rect.top - height - 4 >= 8) top = rect.top - height - 4
    menuStyle.value = { top: `${top}px`, left: `${left}px`, width: `${width}px` }
}

function onFocus() {
    open.value = true
    nextTick(place)
}

function onInput() {
    open.value = true
    nextTick(place)
}

function onBlur() {
    // Give the option's mousedown a chance to land before the menu disappears.
    setTimeout(() => { open.value = false }, 120)
}

function pick(e) {
    query.value = label(e)
    open.value = false
    emit('pick', e)
}

function clear() {
    query.value = ''
    open.value = false
    emit('clear')
}

function reset() {
    query.value = ''
    open.value = false
}

function close() {
    open.value = false
}

defineExpose({ reset })

onMounted(() => {
    window.addEventListener('resize', close)
    window.addEventListener('scroll', close, true)
})
onBeforeUnmount(() => {
    window.removeEventListener('resize', close)
    window.removeEventListener('scroll', close, true)
})
</script>

<template>
    <div class="relative">
        <input ref="inputEl" v-model="query" type="text" autocomplete="off" :placeholder="placeholder"
            :class="inputClass" @input="onInput" @focus="onFocus" @blur="onBlur" />
        <button v-if="query" type="button" :aria-label="$t('common.clear')" @mousedown.prevent="clear"
            class="absolute right-1 top-1/2 -translate-y-1/2 w-5 h-5 rounded-full bg-emerald-50 text-emerald-700 text-xs leading-none">×</button>
        <Teleport to="body">
            <ul v-if="open && matches.length" :style="menuStyle"
                class="fixed z-[100] min-w-[260px] max-h-56 overflow-auto rounded-lg border border-emerald-200 bg-white shadow-lg">
                <li v-for="e in matches" :key="`emp-${e.id}`" @mousedown.prevent="pick(e)"
                    class="px-3 py-2 text-sm text-emerald-900 cursor-pointer hover:bg-emerald-50">
                    <span class="block font-semibold truncate">{{ nameOf(e) }}</span>
                    <span class="block text-[10px] text-emerald-700/70 truncate mt-0.5">{{ e.department || '-' }}</span>
                </li>
            </ul>
        </Teleport>
    </div>
</template>
