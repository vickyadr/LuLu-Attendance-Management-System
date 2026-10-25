<script setup>
import { useAuthStore } from '~/store/auth'
import { useNotifyStore } from '~/store/notify'
import { SMData } from '~/components/SimpleModal.vue'
import { faUsers, faPlus, faEdit, faTrash, faShieldAlt, faCheck, faX, faInbox, faMagnifyingGlass, faXmark, faRotate } from '@fortawesome/free-solid-svg-icons'

const auth = useAuthStore()
const notify = useNotifyStore()
const config = useRuntimeConfig()
const { t } = useI18n()
const sm_data = new SMData()

useHead({ title: 'LuLu - Users' })
definePageMeta({ middleware: ['get-auth'], layout: 'default' })

// Editing other people's accounts needs the same action permission the server
// enforces, so the UI never offers a button that would come back 403.
const canAdd = computed(() => auth.hasActionAccess('users', 'can_add'))
const canEdit = computed(() => auth.hasActionAccess('users', 'can_edit'))
const canDelete = computed(() => auth.hasActionAccess('users', 'can_delete'))

const users = ref([])
const loading = ref(false)
const search = ref('')
const filteredUsers = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return users.value
  return users.value.filter(u => (u.user_nickname || '').toLowerCase().includes(q) || (u.user_fname || '').toLowerCase().includes(q) || (u.user_lname || '').toLowerCase().includes(q))
})
function clearSearch() { search.value = '' }

// Modal state
const showModal = ref(false)
const modalMode = ref('add') // 'add' or 'edit'
const modalTitle = ref('')
const saving = ref(false)

// Form state
const form = ref({
  user_nickname: '',
  nickname: '',
  password: '',
  fname: '',
  lname: '',
})

// Permissions editing
const editingPermissions = ref({})
const showPermissionsModal = ref(false)
const permissionsUser = ref(null)
const permissionsSaving = ref(false)

const ACTIONS = ['can_view', 'can_add', 'can_edit', 'can_delete']
// i18n keys, not resolved labels: $t() at setup time freezes the locale, so the
// permission matrix would keep the old language until a full page refresh.
const ACTION_LABELS = { can_view: 'user.labelView', can_add: 'common.add', can_edit: 'common.edit', can_delete: 'common.delete' }

const ALL_PAGES = [
  { key: 'dashboard', labelKey: 'user.pageDashboard' },
  { key: 'employee', labelKey: 'user.pageEmployee' },
  { key: 'shift', labelKey: 'user.pageShift' },
  { key: 'report', labelKey: 'user.pageReport' },
  { key: 'controller', labelKey: 'user.pageController' },
  { key: 'settings', labelKey: 'user.pageSettings' },
  { key: 'users', labelKey: 'user.pageUsers' },
  { key: 'leave', labelKey: 'user.pageLeave' },
  { key: 'payroll', labelKey: 'user.pagePayroll' },
  { key: 'field', labelKey: 'user.pageField' },
]

async function fetchUsers() {
  loading.value = true
  try {
    const resp = await apiFetch(`${config.public.apiBase}/user/list`, {
      method: 'GET',
      headers: auth.confHeaders(),
    })
    if (resp.code === 200) {
      users.value = resp.data || []
    } else {
      notify.error(t('user.loadFailed'), resp.message || '')
    }
  } catch (e) {
    notify.error(t('user.networkError'), t('user.backendRunning'))
  }
  loading.value = false
}

function openAddModal() {
  modalMode.value = 'add'
  modalTitle.value = t('user.addTitle')
  form.value = { user_nickname: '', nickname: '', password: '', fname: '', lname: '' }
  showModal.value = true
}

function openEditModal(user) {
  modalMode.value = 'edit'
  modalTitle.value = t('user.editTitle')
  form.value = {
    user_nickname: user.user_nickname,
    nickname: user.user_nickname,
    password: '',
    fname: user.user_fname,
    lname: user.user_lname,
  }
  showModal.value = true
}

async function saveUser() {
  saving.value = true

  const body = {
    nickname: form.value.nickname,
    fname: form.value.fname,
    lname: form.value.lname,
  }

  if (modalMode.value === 'add') {
    body.password = form.value.password
  } else {
    body.user_nickname = form.value.user_nickname
    if (form.value.password) {
      body.password = form.value.password
    }
  }

  try {
    const url = modalMode.value === 'add'
      ? `${config.public.apiBase}/user/add`
      : `${config.public.apiBase}/user/edit`

    const resp = await apiFetch(url, {
      method: 'POST',
      headers: auth.confHeaders(),
      body,
    })

    if (resp.code === 200) {
      const was = modalMode.value === 'add'
      showModal.value = false
      await fetchUsers()
      notify.success(was ? t('user.added') : t('user.updated'), form.value.nickname)
    } else {
      notify.error(t('user.saveFailed'), resp.message || '')
    }
  } catch (e) {
    notify.error(t('user.networkError'))
  }
  saving.value = false
}

async function deleteUser(user) {
  // The server answers 400 "Cannot delete yourself", but offering the click and
  // then refusing it reads as a failure. Hide the control instead.
  if (user.user_nickname === auth.user?.nickname) {
    notify.warning(t('user.deleteSelfLong'), user.user_nickname)
    return
  }
  sm_data.setText(t('user.deleteTitle'), t('user.confirmDelete', { name: user.user_nickname }))
  sm_data.setHelper('del-user', { nickname: user.user_nickname })
  sm_data.showOKCancel()
}
async function handleDeleteOk() {
  const helper = sm_data.getHelper('del-user')
  if (helper) {
    try {
      const resp = await apiFetch(`${config.public.apiBase}/user/delete`, {
        method: 'POST',
        headers: auth.confHeaders(),
        body: { user_nickname: helper.nickname },
      })
      if (resp.code === 200) {
        await fetchUsers()
        notify.success(t('user.deleted'), helper.nickname)
      } else {
        notify.error(t('user.deleteFailed'), resp.message || '')
      }
    } catch (e) {
      notify.error(t('user.networkError'))
    }
  }
  sm_data.clear()
}
function handleDeleteCancel() { sm_data.clear() }

// The API returns the full flag set, but a user can also have no row for a page
// at all, and a brand new user has none of them. Normalise to all-false first so
// the grid never reads a missing key as "granted".
function blankPerm() {
  return { can_view: false, can_add: false, can_edit: false, can_delete: false }
}

function normalisePerm(source) {
  const out = {}
  for (const p of ALL_PAGES) {
    const raw = source?.[p.key]
    out[p.key] = raw === true
      ? { can_view: true, can_add: true, can_edit: true, can_delete: true }
      : {
        can_view: raw?.can_view === true,
        can_add: raw?.can_add === true,
        can_edit: raw?.can_edit === true,
        can_delete: raw?.can_delete === true,
      }
  }
  return out
}

function openPermissionsModal(user) {
  permissionsUser.value = user
  editingPermissions.value = normalisePerm(user.permissions)
  showPermissionsModal.value = true
}

// View is the master switch: a page you cannot read is not a page you can write
// to, so ticking an action ticks view, and unticking view clears the rest.
function toggleView(pageKey) {
  const perm = editingPermissions.value[pageKey]
  const next = !perm.can_view
  editingPermissions.value[pageKey] = {
    can_view: next,
    can_add: next ? perm.can_add : false,
    can_edit: next ? perm.can_edit : false,
    can_delete: next ? perm.can_delete : false,
  }
}

function toggleAction(pageKey, action) {
  const perm = editingPermissions.value[pageKey]
  const next = !perm[action]
  editingPermissions.value[pageKey] = { ...perm, [action]: next, can_view: next ? true : perm.can_view }
}

// Ticking every action together is the common case ("full access to this page").
function toggleAll(pageKey) {
  const perm = editingPermissions.value[pageKey]
  const allOn = ACTIONS.every(a => perm[a])
  editingPermissions.value[pageKey] = Object.fromEntries(
    ACTIONS.map(a => [a, !allOn])
  )
}

// Editing your own permissions is allowed (renaming, password), but revoking your
// own access to user management would be a one-way door: there is no admin level
// and no superuser to restore it. Both view and edit must survive, because this
// very page saves through an endpoint gated on can_edit - dropping that flag
// would succeed once and then lock you out of every later permission change.
// can_add and can_delete stay freely editable; nothing becomes unreachable.
// The server refuses both cases with 400 too; this just avoids offering the click.
const editingSelf = computed(() => permissionsUser.value?.user_nickname === auth.user?.nickname)
const selfLockoutReason = computed(() => {
  if (!editingSelf.value) return ''
  if (editingPermissions.value.users?.can_view !== true) {
    return t('user.selfLockoutView')
  }
  if (editingPermissions.value.users?.can_edit !== true) {
    return t('user.selfLockoutEdit')
  }
  return ''
})
const selfLockoutBlocked = computed(() => selfLockoutReason.value !== '')

async function savePermissions() {
  if (selfLockoutBlocked.value) {
    notify.warning(t('user.permSaveFailed'), selfLockoutReason.value)
    return
  }
  permissionsSaving.value = true
  try {
    const perms = Object.entries(editingPermissions.value).map(([page_key, flags]) => ({
      page_key,
      can_view: flags.can_view,
      can_add: flags.can_add,
      can_edit: flags.can_edit,
      can_delete: flags.can_delete,
    }))
    const resp = await apiFetch(`${config.public.apiBase}/user/permissions`, {
      method: 'POST',
      headers: auth.confHeaders(),
      body: {
        user_nickname: permissionsUser.value.user_nickname,
        permissions: perms,
      },
    })
    if (resp.code === 200) {
      const who = permissionsUser.value.user_nickname
      showPermissionsModal.value = false
      await fetchUsers()
      notify.success(t('user.permUpdated'), who)
    } else {
      notify.error(t('user.saveFailed'), resp.message || '')
    }
  } catch (e) {
    notify.error(t('user.networkError'))
  }
  permissionsSaving.value = false
}

// Count pages the user can actually open, not raw row count: a row with every
// flag off grants nothing.
function getPermCount(user) {
  const perms = user.permissions || {}
  return ALL_PAGES.filter(p => permAllows(perms[p.key], 'can_view')).length
}

function permAllows(raw, flag) {
  if (raw === true) return true
  return raw?.[flag] === true
}

// Compact "V A E D" summary so the table shows what a user can do without a modal.
function permSummary(user) {
  const perms = user.permissions || {}
  return ALL_PAGES.map(p => {
    const raw = perms[p.key]
    if (!permAllows(raw, 'can_view')) return null
    const letters = ['V', 'A', 'E', 'D'].filter((_, i) =>
      permAllows(raw, ['can_view', 'can_add', 'can_edit', 'can_delete'][i]))
    // Abbreviation from the stable page key, not the label: compact mono
    // badge stays readable and does not shift with the locale.
    return `${p.key.slice(0, 4).toUpperCase()}:${letters.join('')}`
  }).filter(Boolean)
}

onMounted(() => {
  fetchUsers()
})
</script>

<template>
  <div class="flex flex-col gap-4">

    <!-- page head: plain title row, never a card -->
    <div class="flex flex-col sm:flex-row sm:items-center gap-3">
      <div class="min-w-0">
        <h1 class="display text-[20px] text-text">{{ $t('user.subtitle') }}</h1>
        <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
          <span class="chip chip-quiet">{{ users.length }} {{ $t('user.stAccounts') }}</span>
          <span class="chip chip-emerald">{{ $t('user.manageAccess') }}</span>
        </div>
      </div>
      <div class="sm:ml-auto shrink-0">
        <button v-if="canAdd" @click="openAddModal" class="btn btn-primary w-full sm:w-auto">
          <font-awesome :icon="faPlus" class="w-3.5 h-3.5" />
          <span>{{ $t('user.addTitle') }}</span>
        </button>
      </div>
    </div>

    <!-- controls -->
    <div class="panel p-3 sm:p-4">
      <div class="flex flex-col sm:flex-row gap-2 sm:items-center">
        <div class="relative w-full flex-1 min-w-[240px] max-w-[440px]">
          <font-awesome :icon="faMagnifyingGlass"
            class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
          <input v-model="search" type="text" :placeholder="$t('user.searchPh')"
            class="field has-icon-left has-icon-right" :aria-label="$t('user.searchPh')">
          <button v-if="search" @click="search = ''"
            class="absolute right-1 top-1/2 -translate-y-1/2 w-10 h-10 grid place-content-center rounded-lg text-text-muted hover:bg-surface-raised transition"
            :aria-label="$t('common.clearSearch')">
            <font-awesome :icon="faXmark" class="w-3 h-3" />
          </button>
        </div>

        <button v-if="search" @click="clearSearch()" class="btn btn-secondary">{{ $t('common.reset') }}</button>

        <span class="mono text-[10.5px] text-text-muted sm:ml-auto">
          {{ filteredUsers.length }} / {{ users.length }} {{ $t('user.stAccounts') }}
        </span>
      </div>
    </div>

    <!-- accounts -->
    <div class="panel overflow-hidden">
      <div class="panel-head">
        <span class="inline-flex items-center gap-2 min-w-0">
          <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
          <span class="truncate">{{ $t('user.listTitle') }}</span>
        </span>
        <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted shrink-0">
          {{ filteredUsers.length }} / {{ users.length }}
        </span>
      </div>

      <!-- loading skeleton -->
      <div v-if="loading" class="p-3 space-y-2">
        <div v-for="i in 3" :key="i" class="skeleton h-14 rounded-xl"></div>
      </div>

      <!-- desktop table -->
      <div v-else class="hidden md:block table-shell max-h-[56vh]">
        <table class="w-full min-w-[720px] table-fixed">
          <thead class="sticky top-0 z-10">
            <tr>
              <th class="th w-[22%]">{{ $t('user.colNickname') }}</th>
              <th class="th w-[26%]">{{ $t('user.colName') }}</th>
              <th class="th w-[34%]">{{ $t('user.colAccess') }}</th>
              <th class="th !text-right w-[18%]">{{ $t('user.colActions') }}</th>
            </tr>
          </thead>

          <tbody v-if="filteredUsers.length > 0">
            <tr v-for="user in filteredUsers" :key="user.user_id" class="even:bg-surface-raised/25">
              <td class="td">
                <div class="flex items-center gap-2.5 min-w-0">
                  <span
                    class="w-8 h-8 rounded-lg bg-surface-raised text-primary-deeper grid place-content-center shrink-0">
                    <font-awesome :icon="faUsers" class="w-3.5 h-3.5" />
                  </span>
                  <div class="min-w-0">
                    <p class="text-[13px] font-semibold text-text truncate">{{ user.user_nickname }}</p>
                    <p class="mono text-[10px] text-text-muted truncate">#{{ user.user_id }}</p>
                  </div>
                </div>
              </td>

              <td class="td">
                <p class="text-[13px] font-semibold text-text truncate">{{ user.user_fname }} {{ user.user_lname }}</p>
                <p class="mono text-[10px] text-text-muted truncate">
                  {{ user.user_nickname === auth.user?.nickname ? $t('user.deleteSelf') : $t('user.stAccounts') }}
                </p>
              </td>

              <td class="td">
                <button @click="openPermissionsModal(user)" class="text-left block w-full group">
                  <span class="chip" :class="getPermCount(user) === ALL_PAGES.length ? 'chip-emerald' : 'chip-quiet'">{{
                    $t('user.permPages', { granted: getPermCount(user), total: ALL_PAGES.length }) }}</span>
                  <span v-if="permSummary(user).length"
                    class="block mono text-[10px] text-text-muted mt-1 leading-tight truncate">
                    {{ permSummary(user).join(' · ') }}
                  </span>
                  <span v-else class="block mono text-[10px] text-text-muted/70 mt-1">{{ $t('user.noAccess') }}</span>
                </button>
              </td>

              <td class="td">
                <div class="flex items-center justify-end gap-1">
                  <button @click="openPermissionsModal(user)"
                    class="w-9 h-9 rounded-lg border border-border text-text-muted hover:bg-surface-raised hover:text-primary-deeper transition grid place-content-center"
                    :title="$t('user.permissionsTitle')" :aria-label="$t('user.permissionsTitle')">
                    <font-awesome :icon="faShieldAlt" class="w-3.5 h-3.5" />
                  </button>
                  <button v-if="canEdit" @click="openEditModal(user)"
                    class="w-9 h-9 rounded-lg border border-border text-text-muted hover:bg-surface-raised hover:text-primary-deeper transition grid place-content-center"
                    :title="$t('common.edit')" :aria-label="$t('common.edit')">
                    <font-awesome :icon="faEdit" class="w-3.5 h-3.5" />
                  </button>
                  <button v-if="canDelete && user.user_nickname !== auth.user?.nickname" @click="deleteUser(user)"
                    class="w-9 h-9 rounded-lg border border-border text-danger hover:bg-danger-bg transition grid place-content-center"
                    :title="$t('common.delete')" :aria-label="$t('common.delete')">
                    <font-awesome :icon="faTrash" class="w-3.5 h-3.5" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>

          <tbody v-else>
            <tr>
              <td colspan="4" class="p-0">
                <div class="state">
                  <div class="state-icon">
                    <span class="hud-corner hud-corner-tl"></span>
                    <span class="hud-corner hud-corner-br"></span>
                    <font-awesome :icon="faInbox" class="w-5 h-5" />
                  </div>
                  <p class="state-title">{{ $t('user.noUsers') }}</p>
                  <p class="state-hint">{{ $t('user.addHint') }}</p>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- mobile cards -->
      <div v-if="!loading && filteredUsers.length > 0" class="md:hidden p-3 space-y-2.5">
        <div v-for="user in filteredUsers" :key="`m-${user.user_id}`"
          class="rounded-xl border border-border bg-surface p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-2.5 min-w-0">
              <span class="w-8 h-8 rounded-lg bg-surface-raised text-primary-deeper grid place-content-center shrink-0">
                <font-awesome :icon="faUsers" class="w-3.5 h-3.5" />
              </span>
              <div class="min-w-0">
                <p class="text-[13px] font-semibold text-text truncate">{{ user.user_nickname }}</p>
                <p class="mono text-[10px] text-text-muted truncate">
                  #{{ user.user_id }} <span class="opacity-60">·</span> {{ user.user_fname }} {{ user.user_lname }}
                </p>
              </div>
            </div>
            <span class="chip shrink-0"
              :class="getPermCount(user) === ALL_PAGES.length ? 'chip-emerald' : 'chip-quiet'">{{ getPermCount(user)
              }}/{{ ALL_PAGES.length }}</span>
          </div>

          <p v-if="permSummary(user).length" class="mt-2 mono text-[10px] text-text-muted truncate">{{
            permSummary(user).join(' · ') }}</p>
          <p v-else class="mt-2 mono text-[10px] text-text-muted/70">{{ $t('user.noAccess') }}</p>

          <div class="mt-2.5 flex flex-wrap gap-1.5">
            <button @click="openPermissionsModal(user)" class="btn btn-secondary flex-1">{{ $t('user.colAccess')
              }}</button>
            <button v-if="canEdit" @click="openEditModal(user)" class="btn btn-primary flex-1">{{ $t('common.edit')
              }}</button>
            <button v-if="canDelete && user.user_nickname !== auth.user?.nickname" @click="deleteUser(user)"
              class="btn btn-danger flex-1">{{ $t('common.delete') }}</button>
          </div>
        </div>
      </div>

      <div v-else-if="!loading && filteredUsers.length === 0" class="md:hidden">
        <div class="state">
          <div class="state-icon">
            <span class="hud-corner hud-corner-tl"></span>
            <span class="hud-corner hud-corner-br"></span>
            <font-awesome :icon="faInbox" class="w-5 h-5" />
          </div>
          <p class="state-title">{{ $t('user.noUsers') }}</p>
          <p class="state-hint">{{ $t('user.addHint') }}</p>
        </div>
      </div>

      <div
        class="px-4 py-2.5 border-t border-border flex items-center justify-between gap-2 mono text-[10px] text-text-muted">
        <span>{{ users.length }} {{ $t('user.stAccounts') }}</span>
        <span class="hidden sm:inline">{{ $t('user.footerHint') }}</span>
      </div>
    </div>

    <!-- add / edit account -->
    <div v-if="showModal" class="fixed inset-0 z-[100] flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-black/40 backdrop-blur-[2px]" @click="showModal = false"></div>
      <div class="relative w-full max-w-md panel shadow-tint-lg overflow-hidden">
        <div class="panel-head">
          <span class="truncate">{{ modalTitle }}</span>
          <span class="chip chip-quiet shrink-0">{{ modalMode === 'add' ? $t('common.newModeShort') :
            $t('common.editMode') }}</span>
        </div>

        <div class="panel-body space-y-3">
          <div>
            <label class="label" for="user-nickname">{{ $t('user.colNickname') }}</label>
            <input id="user-nickname" v-model="form.nickname" type="text" class="field"
              :placeholder="$t('user.phNickname')">
          </div>

          <div>
            <label class="label" for="user-password">
              {{ $t('login.password') }} <span v-if="modalMode === 'edit'"
                class="normal-case tracking-normal font-medium text-text-muted">{{ $t('user.passwordKeep') }}</span>
            </label>
            <input id="user-password" v-model="form.password" type="password" class="field" placeholder="********">
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label class="label" for="user-fname">{{ $t('employee.firstName') }}</label>
              <input id="user-fname" v-model="form.fname" type="text" class="field" :placeholder="$t('user.phFirst')">
            </div>
            <div>
              <label class="label" for="user-lname">{{ $t('employee.lastName') }}</label>
              <input id="user-lname" v-model="form.lname" type="text" class="field" :placeholder="$t('user.phLast')">
            </div>
          </div>
        </div>

        <div
          class="px-4 py-3 border-t border-border bg-canvas flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
          <button @click="saveUser" :disabled="saving" class="btn btn-primary flex-1 sm:flex-none">
            <font-awesome v-if="saving" :icon="faRotate" class="w-3.5 h-3.5 animate-spin" />
            {{ saving ? $t('common.saving') : $t('common.save') }}
          </button>
          <button @click="showModal = false" class="btn btn-secondary flex-1 sm:flex-none">{{ $t('common.cancel')
            }}</button>
        </div>
      </div>
    </div>

    <!-- page permissions -->
    <div v-if="showPermissionsModal" class="fixed inset-0 z-[100] flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-black/40 backdrop-blur-[2px]" @click="showPermissionsModal = false"></div>
      <div class="relative w-full max-w-lg panel shadow-tint-lg overflow-hidden flex flex-col max-h-[88vh]">
        <div class="panel-head">
          <span class="inline-flex items-center gap-2 min-w-0">
            <font-awesome :icon="faShieldAlt" class="w-3.5 h-3.5 text-primary shrink-0" />
            <span class="truncate">{{ $t('user.pagePermissions') }}</span>
          </span>
          <span class="chip chip-quiet shrink-0 normal-case tracking-normal">{{ permissionsUser?.user_nickname }}</span>
        </div>

        <div class="panel-body space-y-3 overflow-y-auto">
          <p class="mono text-[10px] tracking-[0.12em] text-text-muted">{{ $t('user.permLegend') }}</p>

          <!-- one row per page, view is the master switch -->
          <div class="space-y-2">
            <div v-for="page in ALL_PAGES" :key="page.key" class="rounded-xl border px-3 py-2.5" :class="[
              editingPermissions[page.key].can_view ? 'border-border bg-surface-raised' : 'border-border bg-surface',
              editingSelf && page.key === 'users' && selfLockoutBlocked ? '!border-danger ring-1 ring-danger/30' : ''
            ]">
              <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                <span class="text-[12.5px] font-semibold text-text truncate">{{ $t(page.labelKey) }}</span>
                <div class="flex items-center gap-1 sm:shrink-0">
                  <button @click="toggleAll(page.key)" :title="$t('user.toggleAll')"
                    class="flex-1 sm:flex-none sm:w-11 h-11 px-2 rounded-lg border text-[10px] font-bold mono transition"
                    :class="ACTIONS.every(a => editingPermissions[page.key][a])
                      ? 'bg-primary-deep text-white border-primary-deep'
                      : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">{{ $t('common.all')
                    }}</button>
                  <button v-for="(action, i) in ACTIONS" :key="action"
                    @click="action === 'can_view' ? toggleView(page.key) : toggleAction(page.key, action)"
                    :title="$t(ACTION_LABELS[action])"
                    class="flex-1 sm:flex-none sm:w-11 h-11 rounded-lg border text-[10px] font-bold mono transition"
                    :class="editingPermissions[page.key][action]
                      ? 'bg-primary-deep text-white border-primary-deep'
                      : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">{{
                        ['V','A','E','D'][i] }}</button>
                </div>
              </div>
            </div>
          </div>

          <div v-if="selfLockoutBlocked"
            class="rounded-xl bg-danger-bg border border-danger/30 px-3.5 py-3 text-[12px] text-danger">
            {{ selfLockoutReason }}
          </div>
        </div>

        <div
          class="px-4 py-3 border-t border-border bg-canvas flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
          <button @click="savePermissions" :disabled="permissionsSaving || selfLockoutBlocked"
            class="btn btn-primary flex-1 sm:flex-none">
            <font-awesome v-if="permissionsSaving" :icon="faRotate" class="w-3.5 h-3.5 animate-spin" />
            {{ permissionsSaving ? $t('common.saving') : $t('common.save') }}
          </button>
          <button @click="showPermissionsModal = false" class="btn btn-secondary flex-1 sm:flex-none">{{
            $t('common.cancel') }}</button>
        </div>
      </div>
    </div>
  </div>
  <SimpleModal :options="sm_data.get()" @ok="handleDeleteOk" @cancel="handleDeleteCancel" v-show="sm_data.isShow()" />
</template>
