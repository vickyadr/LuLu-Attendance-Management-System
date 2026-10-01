<script setup>
import { useChecker, useFormater } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useShiftStore } from '~/store/shift';

const auth = useAuthStore(),
    shift_store = useShiftStore(),
    config = useRuntimeConfig(),
    format = useFormater(),
    check = useChecker(),
    shift = reactive({
        id: 0,
        name: '',
        start_time_str: '08:00',
        end_time_str: '17:00',
        start_enroll_min: 30,
        end_enroll_min: 30,
    }),
    validator = reactive({
        name: '', start_time: '', end_time: '', start_enroll: '', end_enroll: '',
    }),
    emit = defineEmits(['done-edit', 'notif']);

const isEdit = computed(() => shift.id > 0)
const submitting = ref(false)

function callbackDoneEdit() { shift.id = 0; emit('done-edit') }
function callbackNotif(data) { emit('notif', data) }

// --- helpers time ---
function secToTimeStr(sec) {
    let s = ((sec % 86400) + 86400) % 86400
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
    return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`
}
function timeStrToSec(str) {
    if (!str || !str.includes(':')) return 0
    const [h, m] = str.split(':').map(Number)
    return (Number.isFinite(h) ? h : 0) * 3600 + (Number.isFinite(m) ? m : 0) * 60
}
function fmtHM(sec) {
    const s = ((sec % 86400) + 86400) % 86400
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
    return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`
}

const startSec = computed(() => timeStrToSec(shift.start_time_str))
const endSec = computed(() => timeStrToSec(shift.end_time_str))
const durationMin = computed(() => {
    let d = endSec.value - startSec.value
    if (d <= 0) d += 86400
    return Math.round(d / 60)
})
const durationLabel = computed(() => {
    const h = Math.floor(durationMin.value / 60), m = durationMin.value % 60
    return `${h}h ${m}m`
})

// presets
const presets = [
    { label: 'Pagi', sub: '08:00 → 17:00', s: '08:00', e: '17:00', enroll: 30 },
    { label: 'Siang', sub: '14:00 → 22:00', s: '14:00', e: '22:00', enroll: 30 },
    { label: 'Malam', sub: '22:00 → 06:00', s: '22:00', e: '06:00', enroll: 30 },
    { label: 'Fleksibel', sub: '07:00 → 15:00', s: '07:00', e: '15:00', enroll: 15 },
]
function applyPreset(p) {
    shift.start_time_str = p.s
    shift.end_time_str = p.e
    shift.start_enroll_min = p.enroll
    shift.end_enroll_min = p.enroll
}

async function add_shift() {
    submitting.value = true
    validator.name = ''; validator.start_time = ''; validator.end_time = ''; validator.start_enroll = ''; validator.end_enroll = ''
    try {
        const body = {
            name: shift.name.trim(),
            start_time: timeStrToSec(shift.start_time_str),
            end_time: timeStrToSec(shift.end_time_str),
            start_enroll: Number(shift.start_enroll_min) * 60,
            end_enroll: Number(shift.end_enroll_min) * 60,
        }
        if (!body.name || body.name.length < 3) { validator.name = 'Name must be at least 3 characters'; submitting.value = false; return }
        const response = await $fetch(`${config.public.apiBase}/shift/add`, { body, method: 'POST', headers: auth.confHeaders() })
        if (response.code == 200) shift_store.addList(response.data[0]);
        else {
            validator.end_enroll = response.data?.end_enroll || ''
            validator.end_time = response.data?.end_time || ''
            validator.name = response.data?.name || ''
            validator.start_enroll = response.data?.start_enroll || ''
            validator.start_time = response.data?.start_time || ''
        }
        callbackNotif({ title: 'Add Shift', message: response.message })
        if (response.code == 200) { shift.name = ''; /* keep times */ }
    } catch (e) {
        const msg = e?.data?.message || e?.message || 'Connection failed'
        callbackNotif({ title: 'Error', message: msg })
    } finally { submitting.value = false }
}

async function edit_shift() {
    submitting.value = true
    validator.name = ''; validator.start_time = ''; validator.end_time = ''; validator.start_enroll = ''; validator.end_enroll = ''
    try {
        const body = {
            id: shift.id,
            name: shift.name.trim(),
            start_time: timeStrToSec(shift.start_time_str),
            end_time: timeStrToSec(shift.end_time_str),
            start_enroll: Number(shift.start_enroll_min) * 60,
            end_enroll: Number(shift.end_enroll_min) * 60,
        }
        const response = await $fetch(`${config.public.apiBase}/shift/edit`, { body, method: 'POST', headers: auth.confHeaders() })
        if (response.code == 200) {
            shift_store.updateList(response.data[0]);
            shift.id = 0; callbackDoneEdit();
        } else {
            validator.end_enroll = response.data?.end_enroll || ''
            validator.end_time = response.data?.end_time || ''
            validator.name = response.data?.name || ''
            validator.start_enroll = response.data?.start_enroll || ''
            validator.start_time = response.data?.start_time || ''
        }
        callbackNotif({ title: 'Edit Shift', message: response.message })
    } catch (e) {
        callbackNotif({ title: 'Error', message: e?.data?.message || e?.message || 'Connection failed' })
    } finally { submitting.value = false }
}

async function del_shift(id) {
    try {
        const response = await $fetch(`${config.public.apiBase}/shift/delete/${id}`, { method: 'GET', headers: auth.confHeaders() })
        if (response.code == 200) shift_store.removeList(id)
        callbackNotif({ title: 'Delete Shift', message: response.message })
    } catch (e) {
        callbackNotif({ title: 'Error', message: e?.data?.message || e?.message })
    }
}

function fillForm(data) {
    shift.id = data.id
    shift.name = data.name || ''
    // data.start_time / end_time are seconds; start_enroll/end_enroll are absolute seconds
    const st = Number(data.start_time ?? 0)
    const et = Number(data.end_time ?? 0)
    const se = Number(data.start_enroll ?? st)
    const ee = Number(data.end_enroll ?? et)
    shift.start_time_str = secToTimeStr(st)
    shift.end_time_str = secToTimeStr(et)
    // offset minutes
    let offStart = Math.round((st - se) / 60)
    let offEnd = Math.round((ee - et) / 60)
    // normalize negative/overflow to positive minutes within day
    if (offStart < 0) offStart = Math.round(((st + 86400 - se) % 86400) / 60)
    if (offEnd < 0) offEnd = Math.round(((ee - et + 86400) % 86400) / 60)
    // cap to sensible
    shift.start_enroll_min = Math.min(180, Math.max(0, offStart || 30))
    shift.end_enroll_min = Math.min(180, Math.max(0, offEnd || 30))
}

const shift_action = async () => { if (shift.id > 0) await edit_shift(); else await add_shift(); }

// styles for timeline preview (avoid complex inline template literals that break vue compiler)
const enrollStartStyle = computed(() => {
    const start = ((startSec.value - shift.start_enroll_min * 60) % 86400 + 86400) % 86400
    const leftPct = (start / 86400) * 100
    const wPct = (shift.start_enroll_min * 60 / 86400) * 100
    return { left: leftPct + '%', width: wPct + '%' }
})
const workStyle = computed(() => {
    const leftPct = (startSec.value / 86400) * 100
    const wPct = (durationMin.value * 60 / 86400) * 100
    return { left: leftPct + '%', width: wPct + '%' }
})
const enrollEndStyle = computed(() => {
    const leftPct = (endSec.value / 86400) * 100
    const wPct = (shift.end_enroll_min * 60 / 86400) * 100
    return { left: leftPct + '%', width: wPct + '%' }
})

defineExpose({ del_shift, edit_shift, fillForm })
</script>

<template>
    <div class="text-left">
        <!-- header -->
        <div
            class="px-5 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/60 to-lime-50/40 flex items-center justify-between tech-grid-soft">
            <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                <h3 class="text-sm font-bold tracking-wide text-green-900 uppercase mono">{{ isEdit ? 'Edit Shift' :
                    'Create Shift' }}</h3>
                <span class="hidden sm:inline-flex chip chip-emerald mono !text-[9px]">WORK HOURS • PUNCH WINDOW</span>
            </div>
            <span
                class="mono text-[10px] tracking-widest px-2 py-1 rounded-full border text-emerald-700 bg-white border-emerald-200"
                :class="isEdit ? '!bg-amber-500 !text-white !border-amber-500' : ''">{{ isEdit ? 'EDIT MODE' : 'CREATE'
                }}</span>
        </div>

        <form @submit.prevent="shift_action" class="px-5 py-4 space-y-4">
            <!-- Nama -->
            <div>
                <label for="shift_name_new"
                    class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Shift
                    Name <span class="text-red-500">*</span></label>
                <input id="shift_name_new" v-model="shift.name" type="text" required
                    placeholder="e.g. Morning 08–17, Afternoon, Night, Warehouse A Shift"
                    class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30 transition">
                <p v-if="validator.name" class="mt-1.5 text-xs font-medium text-red-500 ml-1">{{ validator.name }}</p>
                <p v-else class="mt-1.5 mono text-[10px] tracking-wide text-emerald-700/40 ml-1">Unique name</p>
            </div>

            <!-- Presets -->
            <div class="rounded-xl bg-white border border-emerald-100 p-2.5">
                <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 mb-2">QUICK PRESETS</p>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                    <button v-for="p in presets" :key="p.label" type="button" @click="applyPreset(p)"
                        class="text-left rounded-xl border-2 px-3 py-2.5 hover:border-emerald-300 hover:bg-emerald-50/60 transition bg-white border-emerald-100">
                        <span class="block text-xs font-bold text-green-900">{{ p.label }}</span>
                        <span class="block mono text-[10px] text-emerald-700/60">{{ p.sub }}</span>
                    </button>
                </div>
            </div>

            <!-- Jam kerja -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                    <label for="shift_start"
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Clock
                        In <span class="text-red-500">*</span></label>
                    <input id="shift_start" v-model="shift.start_time_str" type="time" step="60"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 mono">
                    <p v-if="validator.start_time" class="mt-1 text-xs text-red-500">{{ validator.start_time }}</p>
                </div>
                <div>
                    <label for="shift_end"
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Clock
                        Out <span class="text-red-500">*</span></label>
                    <input id="shift_end" v-model="shift.end_time_str" type="time" step="60"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 mono">
                    <p v-if="validator.end_time" class="mt-1 text-xs text-red-500">{{ validator.end_time }}</p>
                </div>
            </div>

            <!-- Enroll window -->
            <div class="hud-frame glass rounded-2xl border border-emerald-100 p-3.5">
                <div class="flex items-center justify-between mb-3">
                    <span class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700">PUNCH WINDOW
                        (TOLERANCE)</span>
                    <span class="mono text-[10px] tracking-wide text-emerald-700/50">Work duration {{ durationLabel
                        }}</span>
                </div>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-xs font-medium text-green-900 mb-1 ml-1">Earliest clock-in <span
                                class="text-emerald-700/60 font-normal">— before shift start</span></label>
                        <div class="relative">
                            <select v-model.number="shift.start_enroll_min"
                                class="w-full px-3.5 h-11 pr-10 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                                <option :value="0">0 min — exactly on time</option>
                                <option :value="15">15 min before</option>
                                <option :value="30">30 min before</option>
                                <option :value="60">60 min before</option>
                                <option :value="90">90 min before</option>
                                <option :value="120">2 h before</option>
                            </select>
                        </div>
                        <p v-if="validator.start_enroll" class="mt-1 text-xs text-red-500">{{ validator.start_enroll }}
                        </p>
                        <p class="mt-1 mono text-[10px] text-emerald-700/40">Window: {{ fmtHM(startSec -
                            shift.start_enroll_min*60) }} → {{ fmtHM(startSec) }}</p>
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-green-900 mb-1 ml-1">Latest clock-out <span
                                class="text-emerald-700/60 font-normal">— after shift end</span></label>
                        <div class="relative">
                            <select v-model.number="shift.end_enroll_min"
                                class="w-full px-3.5 h-11 pr-10 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                                <option :value="0">0 min — exactly on time</option>
                                <option :value="15">15 min after</option>
                                <option :value="30">30 min after</option>
                                <option :value="60">60 min after</option>
                                <option :value="90">90 min after</option>
                                <option :value="120">2 h after</option>
                            </select>
                        </div>
                        <p v-if="validator.end_enroll" class="mt-1 text-xs text-red-500">{{ validator.end_enroll }}</p>
                        <p class="mt-1 mono text-[10px] text-emerald-700/40">Window: {{ fmtHM(endSec) }} → {{
                            fmtHM(endSec + shift.end_enroll_min*60) }}</p>
                    </div>
                </div>

                <!-- timeline preview -->
                <div class="mt-4 rounded-xl bg-emerald-50/50 border border-emerald-100 px-3 py-3">
                    <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 mb-2">24-HOUR TIMELINE
                        PREVIEW</p>
                    <div class="relative h-8 rounded-full bg-white border border-emerald-100 overflow-hidden flex">
                        <!-- whole day bar -->
                        <div
                            class="absolute inset-y-1.5 left-2 right-2 rounded-full bg-slate-100 border border-slate-200">
                        </div>
                        <!-- enroll start -->
                        <div class="absolute top-1.5 bottom-1.5 rounded-full bg-emerald-100 border border-emerald-200 opacity-80"
                            :style="enrollStartStyle"></div>
                        <!-- work -->
                        <div class="absolute top-1 bottom-1 rounded-full bg-gradient-to-r from-emerald-500 to-emerald-600 shadow-sm border border-emerald-600 flex items-center justify-center"
                            :style="workStyle">
                            <span class="mono text-[9px] font-bold text-white tracking-widest">{{ shift.start_time_str
                                }} → {{ shift.end_time_str }}</span>
                        </div>
                        <!-- enroll end -->
                        <div class="absolute top-1.5 bottom-1.5 rounded-full bg-lime-100 border border-lime-200 opacity-80"
                            :style="enrollEndStyle"></div>
                    </div>
                    <div class="mt-2 flex flex-wrap gap-2 mono text-[10px]">
                        <span class="inline-flex items-center gap-1.5"><span
                                class="w-2 h-2 rounded-full bg-emerald-500"></span> Worked {{ durationLabel }}</span>
                        <span class="inline-flex items-center gap-1.5"><span
                                class="w-2 h-2 rounded-full bg-emerald-200 border border-emerald-300"></span> Masuk -{{
                            shift.start_enroll_min }}m</span>
                        <span class="inline-flex items-center gap-1.5"><span
                                class="w-2 h-2 rounded-full bg-lime-200 border border-lime-300"></span> Pulang +{{
                            shift.end_enroll_min }}m</span>
                    </div>
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-3 pt-1">
                <button type="submit" :disabled="submitting"
                    class="btn-emerald rounded-xl px-8 h-11 text-sm font-bold mono tracking-widest disabled:opacity-60 inline-flex items-center gap-2">
                    <svg v-if="submitting" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                    {{ submitting ? 'Menyimpan...' : (isEdit ? 'Save Changes' : 'Save Shift') }}
                </button>
                <button v-if="isEdit" type="button" @click="callbackDoneEdit()"
                    class="rounded-xl px-6 h-11 text-sm font-medium bg-white border-2 border-slate-200 text-slate-700 hover:bg-slate-50">Cancel</button>
                <button v-else type="button" @click="shift.name = ''"
                    class="rounded-xl px-6 h-11 text-sm font-medium bg-white border-2 border-emerald-100 text-emerald-700 hover:bg-emerald-50">Reset</button>
            </div>
        </form>
    </div>
</template>
