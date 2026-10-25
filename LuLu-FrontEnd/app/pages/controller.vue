<script setup>
import ListController from '~/components/ListController.vue';
import { useChecker } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useDeviceStore } from '~/store/devices';
import { SMData } from '~/components/SimpleModal.vue';
import { useNotifyStore } from '~/store/notify';
import { faLock, faRotate, faChevronDown } from '@fortawesome/free-solid-svg-icons';

const
  config = useRuntimeConfig(),
  check = useChecker(),
  auth = useAuthStore(),
  // Device writes map to the 'controller' page server-side.
  canAdd = computed(() => auth.hasActionAccess('controller', 'can_add')),
  canEdit = computed(() => auth.hasActionAccess('controller', 'can_edit')),
  canDelete = computed(() => auth.hasActionAccess('controller', 'can_delete')),
  device_store = useDeviceStore(),
  device = reactive({
    id: 0,
    name: '',
    sn: '',
    location: '',
    handler: 4,
    timezone: 7,
    ip: '',
  }),
  validator = reactive({
    name: '',
    sn: '',
    location: '',
    handler: '',
    timezone: '',
    ip: '',
  }),
  sm_data = new SMData(),
  notify = useNotifyStore(),
  isEdit = ref(false),
  submitting = ref(false),
  showOtherTz = ref(false);

const isEditMode = computed(() => isEdit.value && device.id > 0)

useHead({ title: "LuLu - Controllers & Devices" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

const listRef = ref(null)
const deviceCount = computed(() => device_store.contents.length)
const onlineCount = computed(() => device_store.contents.filter(d => Number(d.status) === 1).length)
const offlineCount = computed(() => Math.max(0, deviceCount.value - onlineCount.value))
const locationLen = computed(() => (device.location || '').length)

const tzQuick = [
  { v: 7, label: 'WIB', sub: 'GMT+7' },
  { v: 8, label: 'WITA', sub: 'GMT+8' },
  { v: 9, label: 'WIT', sub: 'GMT+9' },
]
const tzOptions = [
  { v: -12, label: 'GMT-12 - Baker Island' },
  { v: -11, label: 'GMT-11 - Pago Pago' },
  { v: -10, label: 'GMT-10 - Hawaii' },
  { v: -9, label: 'GMT-9 - Alaska' },
  { v: -8, label: 'GMT-8 - Los Angeles' },
  { v: -7, label: 'GMT-7 - Denver' },
  { v: -6, label: 'GMT-6 - Chicago' },
  { v: -5, label: 'GMT-5 - New York' },
  { v: -4, label: 'GMT-4 - Santiago' },
  { v: -3, label: 'GMT-3 - Buenos Aires' },
  { v: -2, label: 'GMT-2 - S. Georgia' },
  { v: -1, label: 'GMT-1 - Azores' },
  { v: 0, label: 'GMT+0 - London / UTC' },
  { v: 1, label: 'GMT+1 - Paris, Berlin' },
  { v: 2, label: 'GMT+2 - Cairo' },
  { v: 3, label: 'GMT+3 - Moscow' },
  { v: 4, label: 'GMT+4 - Dubai' },
  { v: 5, label: 'GMT+5 - Karachi' },
  { v: 6, label: 'GMT+6 - Dhaka' },
  { v: 7, label: 'GMT+7 - Jakarta, Bangkok (WIB)' },
  { v: 8, label: 'GMT+8 - Singapore, Bali (WITA)' },
  { v: 9, label: 'GMT+9 - Tokyo, Jayapura (WIT)' },
  { v: 10, label: 'GMT+10 - Sydney' },
  { v: 11, label: 'GMT+11 - Solomon' },
  { v: 12, label: 'GMT+12 - Auckland' },
  { v: 13, label: 'GMT+13 - Samoa' },
  { v: 14, label: 'GMT+14 - Kiritimati' },
]

function tzLabel(v) {
  const o = tzOptions.find(x => x.v === Number(v))
  return o ? o.label : `GMT${Number(v) >= 0 ? '+' : ''}${v}`
}
function isQuickTz(v) { return [7, 8, 9].includes(Number(v)) }

function resetValidator() {
  validator.name = ''; validator.sn = ''; validator.location = ''; validator.handler = ''; validator.timezone = ''; validator.ip = ''
}
function resetForm() {
  device.id = 0; device.name = ''; device.sn = ''; device.location = ''; device.handler = 4; device.timezone = 7; device.ip = ''
  showOtherTz.value = false
  resetValidator()
}
function cancelEdit() {
  isEdit.value = false
  resetForm()
}

function actOk() {
  const h = sm_data.getHelper('act');
  if (!check.isNull(h) && h) {
    if (h.kind == 'del') del_controller(h.id);
  }
  sm_data.clear();
  setTimeout(() => resetValidator(), 2000);
}
function actCancel() { sm_data.clear() }

function actDelete(id) {
  const d = device_store.get(id);
  const sn = d?.sn || `#${id}`
  sm_data.setText($t('device.deleteTitle'), $t('device.deleteConfirm', { sn }));
  sm_data.setHelper('act', { kind: 'del', id });
  sm_data.showOKCancel();
}
function actEdit(id) {
  const d = device_store.get(id);
  if (!d) return
  device.id = id;
  device.name = d.name || '';
  device.sn = d.sn || '';
  device.location = d.location || '';
  device.ip = d.ip || '';
  device.handler = d.h_id ?? d.handler ?? 4;
  device.timezone = Number(d.timezone ?? 7);
  showOtherTz.value = !isQuickTz(device.timezone)
  isEdit.value = true;
  nextTick(() => document.getElementById('device-form-anchor')?.scrollIntoView({ behavior: 'smooth', block: 'start' }))
}

const del_controller = async (id) => {
  if (!canDelete.value) { notify.warning($t('common.noAccess'), $t('device.noPermission')); submitting.value = false; return }
  try {
    const response = await apiFetch(`${config.public.apiBase}/device/delete/${id}`, { method: "GET", headers: auth.confHeaders() });
    if (response.code == 200) device_store.removeList(id);
    notify[response.code == 200 ? 'success' : 'error']($t('device.deleteTitle'), response.message);
  } catch (e) {
    notify.error($t('device.deleteFailed'), e?.data?.message || e.message)
  }
}

const edit_controller = async () => {
  if (!canEdit.value) { notify.warning($t('common.noAccess'), $t('device.noPermission')); submitting.value = false; return }
  submitting.value = true; resetValidator()
  try {
    const response = await apiFetch(`${config.public.apiBase}/device/edit`, {
      body: { id: device.id, name: device.name?.trim(), sn: device.sn?.trim(), location: device.location?.trim(), handler: parseInt(device.handler), timezone: parseInt(device.timezone), ip: device.ip?.trim() },
      method: "POST", headers: auth.confHeaders(),
    })
    if (response.code == 200) {
      device_store.updateList(response.data[0]);
      isEdit.value = false;
      resetForm()
    } else {
      validator.handler = response.data?.handler || '';
      validator.location = response.data?.location || '';
      validator.sn = response.data?.sn || '';
      validator.name = response.data?.name || '';
      validator.timezone = response.data?.timezone || '';
      validator.ip = response.data?.ip || '';
    }
    notify[response.code == 200 ? 'success' : 'error']($t('device.editTitle'), response.message);
  } catch (e) {
    notify.error($t('device.saveFailed'), e?.data?.message || e.message)
  } finally { submitting.value = false }
};

const add_controller = async () => {
  if (!canAdd.value) { notify.warning($t('common.noAccess'), $t('device.noPermission')); submitting.value = false; return }
  submitting.value = true; resetValidator()
  if (!device.name || device.name.trim().length < 2) { validator.name = $t('device.errNameShort'); submitting.value = false; return }
  if (!device.sn || device.sn.trim().length < 3) { validator.sn = $t('device.errSnShort'); submitting.value = false; return }
  try {
    const response = await apiFetch(`${config.public.apiBase}/device/add`, {
      body: { name: device.name?.trim(), sn: device.sn?.trim(), location: device.location?.trim(), handler: parseInt(device.handler), timezone: parseInt(device.timezone), ip: device.ip?.trim() },
      method: "POST", headers: auth.confHeaders(),
    })
    if (response.code == 200) {
      device_store.addList(response.data[0]);
      device.name = ''; device.sn = ''; device.location = ''; device.ip = ''
      resetValidator()
    } else {
      validator.handler = response.data?.handler || '';
      validator.location = response.data?.location || '';
      validator.sn = response.data?.sn || '';
      validator.name = response.data?.name || '';
      validator.timezone = response.data?.timezone || '';
      validator.ip = response.data?.ip || '';
    }
    notify[response.code == 200 ? 'success' : 'error']($t('device.addTitle'), response.message);
  } catch (e) {
    notify.error($t('device.saveFailed'), e?.data?.message || e.message)
  } finally { submitting.value = false }
};

const device_action = async () => {
  if (isEdit.value) await edit_controller();
  else await add_controller();
}
</script>

<template>
  <div class="flex flex-col gap-4">

    <!-- page head -->
    <div class="flex flex-col sm:flex-row sm:items-center gap-3">
      <div class="min-w-0">
        <h1 class="display text-[20px] text-text">{{ $t('device.subtitle') }}</h1>
        <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
          <span class="chip chip-quiet">{{ $t('device.countRegistered', { n: deviceCount }) }}</span>
          <span class="chip chip-ok">
            <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
            {{ onlineCount }} {{ $t('device.stOnline') }}
          </span>
          <span class="chip chip-quiet">{{ offlineCount }} {{ $t('device.stOffline') }}</span>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">

      <!-- list -->
      <div class="lg:col-span-8">
        <div class="panel overflow-hidden">
          <ListController ref="listRef" @edit="actEdit" @delete="actDelete" />
        </div>
      </div>

      <!-- form -->
      <div class="lg:col-span-4 space-y-3">
        <div id="device-form-anchor" class="panel overflow-hidden">
          <div class="panel-head">
            <span class="inline-flex items-center gap-2 min-w-0">
              <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
              <span class="truncate">{{ isEditMode ? $t('device.editModeUp') : $t('device.addModeUp') }}</span>
            </span>
            <span class="chip shrink-0" :class="isEditMode ? 'chip-warn' : 'chip-ok'">{{ isEditMode ?
              $t('device.editMode') : $t('common.newModeShort') }}</span>
          </div>

          <form @submit.prevent="device_action" class="p-4 space-y-4">

            <!-- identity -->
            <section class="rounded-xl bg-surface-raised/60 border border-border p-3.5 space-y-3">
              <p class="flex items-center gap-2 card-label !text-primary-deeper">
                <span
                  class="w-5 h-5 rounded-md bg-primary-deep text-white grid place-content-center text-[10px] font-bold">1</span>
                {{ $t('device.machineIdentity') }}
              </p>

              <div>
                <label for="device_name" class="label">{{ $t('device.fieldName') }} <span
                    class="text-danger">*</span></label>
                <input id="device_name" v-model="device.name" type="text" required
                  :placeholder="$t('device.locationPh')" class="field" :aria-invalid="!!validator.name"
                  :aria-describedby="validator.name ? 'err-device-name' : undefined">
                <p v-if="validator.name" id="err-device-name" role="alert"
                  class="mt-1 text-[11.5px] font-medium text-danger">{{ validator.name }}</p>
              </div>

              <div>
                <label for="serial_number" class="label">{{ $t('device.fieldSn') }} <span
                    class="text-danger">*</span></label>
                <div class="relative">
                  <input id="serial_number" v-model="device.sn" :disabled="isEditMode" type="text" required
                    :placeholder="$t('device.snPh')" class="field mono has-icon-right"
                    :class="isEditMode ? '!bg-warn-bg !border-amber-200 !text-warn cursor-not-allowed' : ''"
                    :aria-invalid="!!validator.sn"
                    :aria-describedby="validator.sn ? 'err-device-sn' : 'hint-device-sn'">
                  <span v-if="isEditMode"
                    class="absolute right-2 top-1/2 -translate-y-1/2 w-6 h-6 rounded-md bg-amber-500 text-white grid place-content-center"
                    :title="$t('device.lockedEdit')">
                    <font-awesome :icon="faLock" class="w-3 h-3" />
                  </span>
                </div>
                <p v-if="validator.sn" id="err-device-sn" role="alert"
                  class="mt-1 text-[11.5px] font-medium text-danger">{{ validator.sn }}</p>
                <p v-else id="hint-device-sn" class="mt-1 text-[10.5px]"
                  :class="isEditMode ? 'text-warn' : 'text-text-muted'">
                  {{ isEditMode ? $t('device.snLocked') : $t('device.snHint') }}
                </p>
              </div>
            </section>

            <!-- connection -->
            <section class="rounded-xl border border-border p-3.5 space-y-3">
              <p class="flex items-center gap-2 card-label !text-primary-deeper">
                <span
                  class="w-5 h-5 rounded-md bg-primary-deeper text-white grid place-content-center text-[10px] font-bold">2</span>
                {{ $t('device.connection') }}
              </p>

              <div>
                <label for="device_ip" class="label">{{ $t('device.ipField') }}</label>
                <input id="device_ip" v-model="device.ip" type="text" :placeholder="$t('device.ipPh')"
                  class="field mono" :aria-invalid="!!validator.ip"
                  :aria-describedby="validator.ip ? 'err-device-ip' : 'hint-device-ip'">
                <p v-if="validator.ip" id="err-device-ip" role="alert"
                  class="mt-1 text-[11.5px] font-medium text-danger">{{ validator.ip }}</p>
                <p v-else id="hint-device-ip" class="mt-1 text-[10.5px] text-text-muted">{{ $t('device.ipHint') }}</p>
              </div>

              <div>
                <p class="label">{{ $t('device.fieldHandler') }} <span class="text-danger">*</span></p>
                <div class="grid grid-cols-2 gap-2">
                  <button type="button" @click="device.handler = 4"
                    class="text-left px-3 py-2.5 rounded-xl border transition"
                    :class="device.handler === 4 ? 'bg-primary-deep border-primary-deep text-white' : 'bg-surface border-border-strong text-text hover:bg-surface-raised'">
                    <span class="block text-[12px] font-bold leading-none">{{ $t('device.handlerIclock') }}</span>
                    <span class="block mono text-[10px] mt-1"
                      :class="device.handler === 4 ? 'text-white/75' : 'text-text-muted'">{{
                        $t('device.handlerRecommended') }}</span>
                  </button>
                  <button type="button" @click="device.handler = 5"
                    class="text-left px-3 py-2.5 rounded-xl border transition"
                    :class="device.handler === 5 ? 'bg-primary-deep border-primary-deep text-white' : 'bg-surface border-border-strong text-text hover:bg-surface-raised'">
                    <span class="block text-[12px] font-bold leading-none">{{ $t('device.protocolZk') }}</span>
                    <span class="block mono text-[10px] mt-1"
                      :class="device.handler === 5 ? 'text-white/75' : 'text-text-muted'">{{
                        $t('device.notFullySupported') }}</span>
                  </button>
                </div>
                <p v-if="validator.handler" class="mt-1 text-[11.5px] text-danger">{{ validator.handler }}</p>
              </div>

              <div>
                <p class="label">{{ $t('device.fieldTimezone') }} <span class="text-danger">*</span></p>
                <div class="flex flex-wrap gap-1.5">
                  <button v-for="q in tzQuick" :key="q.v" type="button"
                    @click="device.timezone = q.v; showOtherTz = false"
                    class="px-3 min-h-[36px] rounded-full border text-[11.5px] font-bold mono transition"
                    :class="device.timezone === q.v ? 'bg-primary-deep border-primary-deep text-white' : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">
                    {{ q.label }}
                  </button>
                  <button type="button" @click="showOtherTz = !showOtherTz"
                    class="px-3 min-h-[36px] rounded-full border text-[11.5px] font-bold mono transition"
                    :class="showOtherTz || !isQuickTz(device.timezone) ? 'bg-primary-deeper border-primary-deeper text-white' : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">
                    {{ $t('device.otherTz') }}
                  </button>
                </div>
                <div v-if="showOtherTz || !isQuickTz(device.timezone)" class="mt-2">
                  <select v-model.number="device.timezone" class="field">
                    <option v-for="o in tzOptions" :key="o.v" :value="o.v">{{ o.label }}</option>
                  </select>
                </div>
                <p v-if="validator.timezone" class="mt-1 text-[11.5px] text-danger">{{ validator.timezone }}</p>
                <p v-else class="mt-1 mono text-[10.5px] text-text-muted">{{ $t('device.selected') }} <b
                    class="text-text">{{ tzLabel(device.timezone) }}</b></p>
              </div>
            </section>

            <!-- location -->
            <div>
              <label for="location" class="label flex items-center gap-2">
                <span>{{ $t('device.fieldLocation') }}</span>
                <span class="ml-auto mono text-[10px] font-normal normal-case tracking-normal"
                  :class="locationLen > 150 ? 'text-warn font-bold' : 'text-text-muted'">{{ locationLen }}/160</span>
              </label>
              <textarea id="location" v-model="device.location" rows="2" maxlength="160"
                :placeholder="$t('device.locationPh2')" class="field"></textarea>
              <p v-if="validator.location" class="mt-1 text-[11.5px] text-danger">{{ validator.location }}</p>
            </div>

            <div class="flex flex-col sm:flex-row gap-2 pt-1">
              <button v-if="isEditMode ? canEdit : canAdd" type="submit" :disabled="submitting"
                class="btn btn-primary flex-1 sm:flex-none sm:px-7">
                <font-awesome v-if="submitting" :icon="faRotate" class="w-3.5 h-3.5 animate-spin" />
                {{ submitting ? $t('common.saving') : (isEditMode ? $t('common.saveChanges') : $t('device.saveDevice'))
                }}
              </button>
              <button v-if="isEditMode" type="button" @click="cancelEdit()"
                class="btn btn-secondary flex-1 sm:flex-none">{{ $t('common.cancel') }}</button>
              <button v-else type="button" @click="resetForm()" class="btn btn-secondary flex-1 sm:flex-none">{{
                $t('common.reset') }}</button>
            </div>
          </form>
        </div>

        <!-- setup help -->
        <details class="panel overflow-hidden group">
          <summary class="px-4 py-3 cursor-pointer list-none flex items-center gap-2 card-label select-none">
            <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
            {{ $t('device.stepsTitle') }}
            <font-awesome :icon="faChevronDown"
              class="w-3 h-3 ml-auto text-text-muted transition group-open:rotate-180" />
          </summary>
          <div class="px-4 pb-3.5 pt-0">
            <ol class="space-y-1.5 text-[11.5px] leading-relaxed text-text-muted list-decimal list-inside">
              <li>{{ $t('device.step1') }}</li>
              <li>{{ $t('device.step2') }}</li>
              <li>{{ $t('device.step3') }}</li>
              <li>{{ $t('device.step4') }}</li>
            </ol>
            <div class="mt-3 flex flex-wrap gap-1.5">
              <span class="chip chip-quiet">{{ $t('device.tzWib') }}</span>
              <span class="chip chip-quiet">{{ $t('device.tzWita') }}</span>
              <span class="chip chip-quiet">{{ $t('device.tzWit') }}</span>
            </div>
          </div>
        </details>
      </div>
    </div>
  </div>

  <SimpleModal :options="sm_data.get()" @ok="actOk" @cancel="actCancel" v-show="sm_data.isShow()" />
</template>
