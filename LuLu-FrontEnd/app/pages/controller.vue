<script setup>
import ListController from '~/components/ListController.vue';
import { useChecker } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useDeviceStore } from '~/store/devices';
import { SMData } from '~/components/SimpleModal.vue';

const
    config = useRuntimeConfig(),
    check = useChecker(),
    auth = useAuthStore(),
    device_store = useDeviceStore(),
    device = reactive({
        id: 0,
        name: '',
        sn: '',
        location: '',
        handler: 4,
        timezone: 7,
    }),
    validator = reactive({
        name: '',
        sn: '',
        location: '',
        handler: '',
        timezone: '',
    }),
    sm_data = new SMData(),
    isEdit = ref(false),
    submitting = ref(false),
    showOtherTz = ref(false);

const isEditMode = computed(() => isEdit.value && device.id > 0)

useHead({ title: "LuLu — Controllers & Devices" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

const listRef = ref(null)
const deviceCount = computed(() => device_store.contents.length)
const onlineCount = computed(() => device_store.contents.filter(d => Number(d.status) === 1).length)
const offlineCount = computed(() => Math.max(0, deviceCount.value - onlineCount.value))
const locationLen = computed(()=> (device.location||'').length)

const tzQuick = [
    { v: 7, label: 'WIB', sub: 'GMT+7' },
    { v: 8, label: 'WITA', sub: 'GMT+8' },
    { v: 9, label: 'WIT', sub: 'GMT+9' },
]
const tzOptions = [
    { v: -12, label: 'GMT-12 — Baker Island' },
    { v: -11, label: 'GMT-11 — Pago Pago' },
    { v: -10, label: 'GMT-10 — Hawaii' },
    { v: -9, label: 'GMT-9 — Alaska' },
    { v: -8, label: 'GMT-8 — Los Angeles' },
    { v: -7, label: 'GMT-7 — Denver' },
    { v: -6, label: 'GMT-6 — Chicago' },
    { v: -5, label: 'GMT-5 — New York' },
    { v: -4, label: 'GMT-4 — Santiago' },
    { v: -3, label: 'GMT-3 — Buenos Aires' },
    { v: -2, label: 'GMT-2 — S. Georgia' },
    { v: -1, label: 'GMT-1 — Azores' },
    { v: 0, label: 'GMT+0 — London / UTC' },
    { v: 1, label: 'GMT+1 — Paris, Berlin' },
    { v: 2, label: 'GMT+2 — Cairo' },
    { v: 3, label: 'GMT+3 — Moscow' },
    { v: 4, label: 'GMT+4 — Dubai' },
    { v: 5, label: 'GMT+5 — Karachi' },
    { v: 6, label: 'GMT+6 — Dhaka' },
    { v: 7, label: 'GMT+7 — Jakarta, Bangkok (WIB)' },
    { v: 8, label: 'GMT+8 — Singapore, Bali (WITA)' },
    { v: 9, label: 'GMT+9 — Tokyo, Jayapura (WIT)' },
    { v: 10, label: 'GMT+10 — Sydney' },
    { v: 11, label: 'GMT+11 — Solomon' },
    { v: 12, label: 'GMT+12 — Auckland' },
    { v: 13, label: 'GMT+13 — Samoa' },
    { v: 14, label: 'GMT+14 — Kiritimati' },
]

function tzLabel(v){
    const o = tzOptions.find(x=> x.v===Number(v))
    return o ? o.label : `GMT${Number(v)>=0?'+':''}${v}`
}
function isQuickTz(v){ return [7,8,9].includes(Number(v)) }

function resetValidator(){
    validator.name=''; validator.sn=''; validator.location=''; validator.handler=''; validator.timezone=''
}
function resetForm(){
    device.id=0; device.name=''; device.sn=''; device.location=''; device.handler=4; device.timezone=7
    showOtherTz.value=false
    resetValidator()
}
function cancelEdit(){
    isEdit.value=false
    resetForm()
}

function actOk(){
    const id = sm_data.getHelper('del');
    if (!check.isNull(id)) del_controller(id);
    sm_data.clear();
    setTimeout(()=> resetValidator(), 2000);
}
function actCancel(){ sm_data.clear() }

function actDelete(id){
    const d = device_store.get(id);
    const sn = d?.sn || `#${id}`
    sm_data.setText("Delete Controller", `Device "${sn}" akan dihapus, yakin?`);
    sm_data.setHelper('del', id);
    sm_data.showOKCancel();
}
function actEdit(id){
    const d = device_store.get(id);
    if (!d) return
    device.id = id;
    device.name = d.name || '';
    device.sn = d.sn || '';
    device.location = d.location || '';
    device.handler = d.h_id ?? d.handler ?? 4;
    device.timezone = Number(d.timezone ?? 7);
    showOtherTz.value = !isQuickTz(device.timezone)
    isEdit.value = true;
    nextTick(()=> document.getElementById('device-form-anchor')?.scrollIntoView({ behavior:'smooth', block:'start' }))
}

const del_controller = async (id) =>{
   try {
       const response = await $fetch(`${config.public.apiBase}/device/delete/${id}`, { method: "GET", headers: auth.confHeaders() });
       if (response.code == 200) device_store.removeList(id);
       sm_data.setText("Delete Device", response.message);
       sm_data.showOK();
   } catch(e){
       sm_data.setText("Error", e?.data?.message || e.message); sm_data.showOK()
   }
}

const edit_controller = async () =>{
    submitting.value=true; resetValidator()
    try {
        const response = await $fetch(`${config.public.apiBase}/device/edit`, {
            body: { id: device.id, name: device.name?.trim(), sn: device.sn?.trim(), location: device.location?.trim(), handler: parseInt(device.handler), timezone: parseInt(device.timezone) },
            method: "POST", headers: auth.confHeaders(),
        })
        if (response.code == 200){
            device_store.updateList(response.data[0]);
            isEdit.value = false;
            resetForm()
        } else {
            validator.handler = response.data?.handler || '';
            validator.location = response.data?.location || '';
            validator.sn = response.data?.sn || '';
            validator.name = response.data?.name || '';
            validator.timezone = response.data?.timezone || '';
        }
        sm_data.setText("Edit Device", response.message); sm_data.showOK();
    } catch(e){
        sm_data.setText("Error", e?.data?.message || e.message); sm_data.showOK()
    } finally { submitting.value=false }
};

const add_controller = async () =>{
    submitting.value=true; resetValidator()
    if (!device.name || device.name.trim().length < 2) { validator.name='Name must be at least 2 characters'; submitting.value=false; return }
    if (!device.sn || device.sn.trim().length < 3) { validator.sn='Serial must be at least 3 characters'; submitting.value=false; return }
    try {
        const response = await $fetch(`${config.public.apiBase}/device/add`, {
            body: { name: device.name?.trim(), sn: device.sn?.trim(), location: device.location?.trim(), handler: parseInt(device.handler), timezone: parseInt(device.timezone) },
            method: "POST", headers: auth.confHeaders(),
        })
        if (response.code == 200){
            device_store.addList(response.data[0]);
            device.name=''; device.sn=''; device.location=''
            resetValidator()
        } else {
            validator.handler = response.data?.handler || '';
            validator.location = response.data?.location || '';
            validator.sn = response.data?.sn || '';
            validator.name = response.data?.name || '';
            validator.timezone = response.data?.timezone || '';
        }
        sm_data.setText("Add Device", response.message); sm_data.showOK();
    } catch(e){
        sm_data.setText("Error", e?.data?.message || e.message); sm_data.showOK()
    } finally { submitting.value=false }
};

const device_action = async () =>{
    if (isEdit.value) await edit_controller();
    else await add_controller();
}
</script>

<template>
    <div class="space-y-4">
        <!-- Header — compact hero -->
        <div class="hud-frame glass rounded-[1.4rem] border border-emerald-100 px-4 sm:px-5 py-4 flex flex-col gap-3 tech-grid-soft relative overflow-hidden">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span>
            <span class="hud-corner hud-corner-br hidden sm:block"></span>
            <div class="flex items-start gap-3 relative">
                <div class="hidden sm:flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm shrink-0">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="4" width="18" height="14" rx="2"/><path d="M7 8h10"/><path d="M8 12h2"/><path d="M12 12h4"/><path d="M8 16h8"/></svg>
                </div>
                <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-center gap-2">
                        <h1 class="display text-[17px] sm:text-lg font-extrabold text-green-900 tracking-tight">Controller & Device</h1>
                        <span class="chip chip-emerald mono !text-[9px]">ADMS • iCLOCK • ZKNET</span>
                        <span class="hidden sm:inline-flex items-center gap-1.5 mono text-[10px] text-emerald-700/50"><span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span> syncs to transactions automatically</span>
                    </div>
                    <p class="text-xs text-emerald-700/60 mt-1 leading-relaxed max-w-3xl">Register a punch machine with its SN. Pick the timezone that matches where the machine sits — reports adjust automatically.</p>
                </div>
                <button @click="listRef?.reload && listRef.reload()" class="hidden sm:inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-emerald-600 text-white mono text-xs font-bold hover:bg-emerald-700 shrink-0 shadow-sm">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-2.64-6.36"/><path d="M21 3v6h-6"/></svg>
                    Reload
                </button>
            </div>
            <!-- Stat pills -->
            <div class="flex flex-wrap gap-2 mono text-[11px]">
                <span class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-white border border-emerald-100 text-green-900 font-bold shadow-sm">
                    <span class="w-2 h-2 rounded-full bg-emerald-500"></span> {{ deviceCount }} registered
                </span>
                <span class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-emerald-600 text-white font-bold shadow-sm">
                    <span class="w-2 h-2 rounded-full bg-white animate-pulse"></span> {{ onlineCount }} online
                </span>
                <span class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-white border border-slate-200 text-slate-600 font-bold">
                    {{ offlineCount }} offline
                </span>
                <span class="hidden sm:inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-full bg-emerald-50 border border-emerald-100 text-emerald-700">Unique SN • WIB/WITA/WIT zone • ADMS handler</span>
                <button @click="listRef?.reload && listRef.reload()" class="sm:hidden ml-auto inline-flex items-center gap-1 px-3 py-1.5 rounded-full bg-white border border-emerald-100 text-emerald-700 font-bold">↻ Load</button>
            </div>
        </div>

        <!-- Main grid -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">
            <!-- LEFT: List — 7 cols -->
            <div class="lg:col-span-7 xl:col-span-7">
                <div class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden bg-white">
                    <ListController ref="listRef" @edit="actEdit" @delete="actDelete" />
                </div>
                <p class="mt-2 mono text-[10px] tracking-wide text-emerald-700/40 px-1 flex flex-wrap gap-1.5">
                    <span>Tips:</span>
                    <span class="px-1.5 py-0.5 rounded-full bg-white border border-emerald-100 text-emerald-700">type the SN in the search box</span>
                    <span class="px-1.5 py-0.5 rounded-full bg-white border border-emerald-100 text-emerald-700">filter Online to check which units are live</span>
                    <span class="hidden sm:inline">• Edit without changing the SN</span>
                </p>
            </div>

            <!-- RIGHT: Form — 5 cols -->
            <div class="lg:col-span-5 xl:col-span-5 space-y-3.5">
                <!-- Form card -->
                <div id="device-form-anchor" class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden bg-white">
                    <div class="px-4 sm:px-5 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/70 via-white to-lime-50/20 flex items-center justify-between gap-2">
                        <div class="flex items-center gap-2 min-w-0">
                            <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse shrink-0"></span>
                            <h2 class="mono text-xs font-extrabold tracking-[0.14em] text-emerald-700 truncate">{{ isEditMode ? 'EDIT DEVICE' : 'ADD DEVICE' }}</h2>
                            <span class="hidden sm:inline-flex chip mono !text-[9px] shrink-0" :class="isEditMode ? 'bg-amber-500 text-white border-amber-500' : 'chip-emerald'">{{ isEditMode ? 'EDIT' : 'NEW' }}</span>
                        </div>
                        <span class="mono text-[10px] tracking-widest text-emerald-700/40 hidden sm:inline shrink-0">Press Enter to save</span>
                        <span v-if="isEditMode" class="sm:hidden chip bg-amber-500 text-white mono !text-[9px] shrink-0">EDIT</span>
                    </div>

                    <form @submit.prevent="device_action" class="px-4 sm:px-5 py-4 space-y-4">
                        <!-- Section: Identitas -->
                        <div class="rounded-xl bg-emerald-50/40 border border-emerald-100 p-3.5 space-y-3.5">
                            <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2">
                                <span class="w-6 h-6 rounded-lg bg-emerald-600 text-white flex items-center justify-center text-[11px]">1</span>
                                MACHINE IDENTITY
                                <span class="ml-auto mono text-[10px] font-normal tracking-wide text-emerald-700/50 hidden sm:inline">any name • SN from the machine sticker</span>
                            </p>
                            <div>
                                <label for="device_name" class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Device Name <span class="text-red-500">*</span></label>
                                <input id="device_name" v-model="device.name" type="text" required placeholder="Contoh: Pintu Utama / Mesin Gudang A" class="w-full px-3.5 h-11 rounded-xl border-2 border-white bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30 transition shadow-sm">
                                <p v-if="validator.name" class="mt-1.5 text-xs font-medium text-red-500 ml-1">{{ validator.name }}</p>
                                <p v-else class="mt-1 mono text-[10px] tracking-wide text-emerald-700/40 ml-1">Name shown on the dashboard & in reports</p>
                            </div>
                            <div>
                                <label for="serial_number" class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Serial Number (SN) <span class="text-red-500">*</span></label>
                                <div class="relative">
                                    <input id="serial_number" v-model="device.sn" :disabled="isEditMode" type="text" required placeholder="Contoh: 583298001234" class="w-full px-3.5 h-11 pr-10 rounded-xl border-2 bg-white outline-none text-sm text-green-900 placeholder:text-green-900/30 mono font-medium tracking-wide transition shadow-sm" :class="isEditMode ? 'border-amber-200 bg-amber-50/50 text-amber-800 cursor-not-allowed' : 'border-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100'">
                                    <span v-if="isEditMode" class="absolute right-2.5 top-1/2 -translate-y-1/2 w-7 h-7 rounded-lg bg-amber-500 text-white flex items-center justify-center shadow-sm" title="Terkunci saat edit">
                                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg>
                                    </span>
                                </div>
                                <p v-if="validator.sn" class="mt-1.5 text-xs font-medium text-red-500 ml-1">{{ validator.sn }}</p>
                                <p v-else class="mt-1 mono text-[10px] tracking-wide ml-1" :class="isEditMode ? 'text-amber-600' : 'text-emerald-700/40'">{{ isEditMode ? 'SN is locked — delete and re-create to swap machines' : 'Find the SN under Menu → Device on the machine • it must be unique' }}</p>
                            </div>
                        </div>

                        <!-- Section: Koneksi -->
                        <div class="rounded-xl bg-white border-2 border-emerald-100 p-3.5 space-y-3.5">
                            <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2">
                                <span class="w-6 h-6 rounded-lg bg-green-700 text-white flex items-center justify-center text-[11px]">2</span>
                                CONNECTION
                            </p>
                            <!-- Handler as segmented -->
                            <div>
                                <label class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Handler <span class="text-red-500">*</span></label>
                                <div class="grid grid-cols-2 gap-2">
                                    <button type="button" @click="device.handler=4" class="text-left px-3 py-2.5 rounded-xl border-2 transition" :class="device.handler===4 ? 'bg-emerald-600 border-emerald-600 text-white shadow-sm' : 'bg-white border-slate-200 text-slate-700 hover:border-emerald-200'">
                                        <p class="text-xs font-bold leading-none">iClock / ADMS</p>
                                        <p class="mono text-[10px] mt-1" :class="device.handler===4 ? 'text-white/80' : 'text-slate-400'">Recommended • stable</p>
                                    </button>
                                    <button type="button" @click="device.handler=5" class="text-left px-3 py-2.5 rounded-xl border-2 transition" :class="device.handler===5 ? 'bg-slate-800 border-slate-800 text-white shadow-sm' : 'bg-white border-slate-200 text-slate-700 hover:border-slate-300'">
                                        <p class="text-xs font-bold leading-none">ZKNET</p>
                                        <p class="mono text-[10px] mt-1" :class="device.handler===5 ? 'text-white/70' : 'text-slate-400'">Not fully supported</p>
                                    </button>
                                </div>
                                <p v-if="validator.handler" class="mt-1 text-xs text-red-500">{{ validator.handler }}</p>
                                <p v-else class="mt-1 mono text-[10px] text-emerald-700/40 ml-1">ADMS for modern machines — ZKNET is legacy only</p>
                            </div>
                            <!-- Timezone quick -->
                            <div>
                                <label class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Timezone <span class="text-red-500">*</span></label>
                                <div class="flex flex-wrap gap-1.5">
                                    <button v-for="q in tzQuick" :key="q.v" type="button" @click="device.timezone=q.v; showOtherTz=false" class="px-3.5 py-2 rounded-full border-2 mono text-xs font-bold transition" :class="device.timezone===q.v ? 'bg-emerald-600 border-emerald-600 text-white shadow-sm' : 'bg-white border-emerald-100 text-emerald-700 hover:border-emerald-300'">
                                        {{ q.label }} <span class="opacity-60 font-normal hidden sm:inline">• {{ q.sub }}</span>
                                    </button>
                                    <button type="button" @click="showOtherTz=!showOtherTz" class="px-3 py-2 rounded-full border-2 mono text-xs font-bold transition" :class="showOtherTz || !isQuickTz(device.timezone) ? 'bg-slate-800 border-slate-800 text-white' : 'bg-white border-slate-200 text-slate-600 hover:border-slate-300'">
                                        Other ▾
                                    </button>
                                </div>
                                <div v-if="showOtherTz || !isQuickTz(device.timezone)" class="mt-2">
                                    <select v-model.number="device.timezone" class="w-full px-3 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                                        <option v-for="o in tzOptions" :key="o.v" :value="o.v">{{ o.label }}</option>
                                    </select>
                                </div>
                                <p v-if="validator.timezone" class="mt-1 text-xs text-red-500">{{ validator.timezone }}</p>
                                <p v-else class="mt-1 mono text-[10px] text-emerald-700/40 ml-1">Selected: <b class="text-emerald-700">{{ tzLabel(device.timezone) }}</b> • pick WIB/WITA/WIT for Indonesia</p>
                            </div>
                        </div>

                        <!-- Section: Location -->
                        <div>
                            <label for="location" class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1 flex items-center gap-2">
                                Location
                                <span class="ml-auto mono text-[10px] font-normal normal-case tracking-wide" :class="locationLen>150 ? 'text-amber-600 font-bold' : 'text-emerald-700/40'">{{ locationLen }}/160</span>
                            </label>
                            <textarea id="location" v-model="device.location" rows="2" maxlength="160" placeholder="Contoh: Lantai 2 — Lobby Utama, Gedung A (opsional)" class="w-full px-3.5 py-2.5 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30 resize-none transition shadow-sm"></textarea>
                            <p v-if="validator.location" class="mt-1 text-xs text-red-500">{{ validator.location }}</p>
                            <p v-else class="mt-1 mono text-[10px] text-emerald-700/40 ml-1">Max 160 characters • shown in monitoring & reports</p>
                        </div>

                        <!-- Actions -->
                        <div class="flex flex-col sm:flex-row gap-2.5 pt-1">
                            <button type="submit" :disabled="submitting" class="flex-1 sm:flex-initial btn-emerald rounded-xl px-6 sm:px-8 py-3 text-sm font-bold mono tracking-widest disabled:opacity-60 inline-flex items-center justify-center gap-2 shadow-sm">
                                <svg v-if="submitting" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/></svg>
                                {{ submitting ? 'Menyimpan…' : (isEditMode ? 'Save Changes' : 'Save Device') }}
                            </button>
                            <button v-if="isEditMode" type="button" @click="cancelEdit()" class="px-6 py-3 rounded-xl bg-white border-2 border-slate-200 text-slate-700 hover:bg-slate-50 text-sm font-bold mono">Cancel</button>
                            <button v-else type="button" @click="resetForm()" class="px-6 py-3 rounded-xl bg-white border-2 border-emerald-100 text-emerald-700 hover:bg-emerald-50 text-sm font-bold mono">Reset</button>
                        </div>
                        <!-- Preview chip -->
                        <div class="flex flex-wrap gap-1.5 mono text-[10px] pt-1 border-t border-emerald-50">
                            <span class="px-2 py-1 rounded-full bg-emerald-50 border border-emerald-100 text-emerald-700">Preview: {{ device.name || '—' }} • SN {{ device.sn || '—' }}</span>
                            <span class="px-2 py-1 rounded-full bg-white border border-slate-200 text-slate-600">{{ tzLabel(device.timezone) }} • {{ device.handler===4 ? 'ADMS' : 'ZKNET' }}</span>
                        </div>
                    </form>
                </div>

                <!-- Help — compact -->
                <details class="rounded-2xl border border-emerald-100 bg-emerald-50/40 overflow-hidden group">
                    <summary class="px-4 py-3 cursor-pointer list-none flex items-center gap-2 mono text-[11px] font-bold tracking-widest text-emerald-700 select-none">
                        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                        REGISTRATION • 4 STEPS
                        <span class="ml-auto text-emerald-700/40 group-open:rotate-180 transition">▾</span>
                    </summary>
                    <div class="px-4 pb-3.5 pt-1">
                        <ol class="space-y-1.5 mono text-[11px] leading-relaxed text-emerald-800/80 list-decimal list-inside">
                            <li>On the machine: <b>Menu → Device Info</b> note it down <b>SN</b>.</li>
                            <li>Enter <b>Nama</b> + <b>SN</b> in the form — pick <b>WIB/WITA/WIT</b>.</li>
                            <li>Pick a handler <b>ADMS</b>  (recommended).</li>
                            <li>Save → check the status in the table on the left <b>Online</b>.</li>
                        </ol>
                        <div class="mt-3 flex flex-wrap gap-1.5">
                            <span class="chip chip-emerald mono !text-[9px]">WIB GMT+7</span>
                            <span class="chip bg-white border-emerald-200 text-emerald-700 border mono !text-[9px]">WITA GMT+8</span>
                            <span class="chip bg-white border-emerald-200 text-emerald-700 border mono !text-[9px]">WIT GMT+9</span>
                        </div>
                    </div>
                </details>
            </div>
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" @ok="actOk" @cancel="actCancel" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>
