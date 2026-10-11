<script setup>
import { useAuthStore } from '~/store/auth'
import { useNotifyStore } from '~/store/notify'
import { faUsers, faPlus, faEdit, faTrash, faShieldAlt, faCheck, faX } from '@fortawesome/free-solid-svg-icons'

const auth = useAuthStore()
const notify = useNotifyStore()
const config = useRuntimeConfig()
const { t } = useI18n()

useHead({ title: 'LuLu - Users' })
definePageMeta({ middleware: ['get-auth'], layout: 'default' })

// Editing other people's accounts needs the same action permission the server
// enforces, so the UI never offers a button that would come back 403.
const canAdd = computed(() => auth.hasActionAccess('users', 'can_add'))
const canEdit = computed(() => auth.hasActionAccess('users', 'can_edit'))
const canDelete = computed(() => auth.hasActionAccess('users', 'can_delete'))

const users = ref([])
const loading = ref(false)

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
const ACTION_LABELS = { can_view: $t('user.labelView'), can_add: $t('common.add'), can_edit: $t('common.edit'), can_delete: $t('common.delete') }

const ALL_PAGES = [
    { key: 'dashboard', label: $t('user.pageDashboard') },
    { key: 'employee', label: $t('user.pageEmployee') },
    { key: 'shift', label: $t('user.pageShift') },
    { key: 'report', label: $t('user.pageReport') },
    { key: 'controller', label: $t('user.pageController') },
    { key: 'settings', label: $t('user.pageSettings') },
    { key: 'users', label: $t('user.pageUsers') },
    { key: 'leave', label: $t('user.pageLeave') },
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
    if (!confirm(t('user.confirmDelete', { name: user.user_nickname }))) return

    try {
        const resp = await apiFetch(`${config.public.apiBase}/user/delete`, {
            method: 'POST',
            headers: auth.confHeaders(),
            body: { user_nickname: user.user_nickname },
        })
        if (resp.code === 200) {
            await fetchUsers()
            notify.success(t('user.deleted'), user.user_nickname)
        } else {
            notify.error(t('user.deleteFailed'), resp.message || '')
        }
    } catch (e) {
        notify.error(t('user.networkError'))
    }
}

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
        return 'You cannot remove your own user management access - that would lock every account out.'
    }
    if (editingPermissions.value.users?.can_edit !== true) {
        return 'You cannot remove your own edit access to user management - you would not be able to change any permission again, including this one.'
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
        return `${p.label.slice(0, 4).toUpperCase()}:${letters.join('')}`
    }).filter(Boolean)
}

onMounted(() => {
    fetchUsers()
})
</script>

<template>
    <div class="space-y-4">
        <!-- Header -->
        <div
            class="hud-frame glass rounded-2xl border border-emerald-100 px-5 py-4 flex flex-col sm:flex-row sm:items-center gap-3 tech-grid-soft shrink-0">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span><span
                class="hud-corner hud-corner-br hidden sm:block"></span>
            <div class="flex items-start gap-3 flex-1 min-w-0">
                <div
                    class="hidden sm:flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm">
                    <font-awesome :icon="faUsers" class="w-[18px] h-[18px]" />
                </div>
                <div class="min-w-0">
                    <h1 class="display text-lg font-extrabold text-green-900 tracking-tight">{{ $t('user.subtitle') }}
                    </h1>
                    <div class="mt-2 flex flex-wrap items-center gap-2 mono text-[10px]">
                        <span class="chip chip-emerald">{{ users.length }} users</span>
                        <span class="chip chip-emerald">{{ $t('user.manageAccess') }}</span>
                    </div>
                </div>
            </div>
            <button v-if="canAdd" @click="openAddModal"
                class="btn-emerald px-5 h-11 rounded-xl text-sm font-bold inline-flex items-center justify-center gap-2 transition shrink-0 w-full sm:w-auto">
                <font-awesome :icon="faPlus" class="w-4 h-4" />
                <span>{{ $t('user.addTitle') }}</span>
            </button>
        </div>

        <!-- Users table -->
        <div class="hud-frame glass rounded-2xl border border-emerald-100 overflow-hidden">
            <div class="px-4 py-2.5 bg-white/60 border-b border-emerald-100 flex items-center justify-between gap-2">
                <span class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2">
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> {{ $t('user.listTitle') }}
                </span>
                <span class="mono text-[11px] text-emerald-700/60">{{ users.length }} shown • {{ users.length }}
                    total</span>
            </div>

            <!-- Loading skeleton -->
            <div v-if="loading" class="p-3 space-y-2 bg-white">
                <div v-for="i in 3" :key="i"
                    class="h-14 rounded-xl bg-emerald-50 animate-pulse border border-emerald-100"></div>
            </div>

            <!-- Desktop table -->
            <div v-else class="hidden md:block overflow-auto max-h-[52vh] lg:max-h-[56vh] bg-white">
                <table class="w-full min-w-[640px] border-collapse">
                    <thead class="sticky top-0 z-10">
                        <tr class="bg-gradient-to-r from-emerald-600 to-green-600 text-white">
                            <th
                                class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[22%]">
                                {{ $t('user.colNickname') }}</th>
                            <th
                                class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[28%]">
                                {{ $t('user.colName') }}</th>
                            <th
                                class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                                {{ $t('user.colAccess') }}</th>
                            <th
                                class="py-2.5 px-3 text-right mono text-[10px] font-bold tracking-[0.14em] uppercase w-[18%]">
                                {{ $t('user.colActions') }}</th>
                        </tr>
                    </thead>
                    <tbody v-if="users.length > 0" class="divide-y divide-emerald-50 bg-white">
                        <tr v-for="(user, idx) in users" :key="user.user_id"
                            class="hover:bg-emerald-50/50 transition text-xs"
                            :class="idx % 2 === 0 ? 'bg-white' : 'bg-emerald-50/10'">
                            <td class="py-3 px-3">
                                <div class="flex items-center gap-2.5">
                                    <span
                                        class="w-9 h-9 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white flex items-center justify-center shrink-0 shadow-sm">
                                        <FontAwesome :icon="faUsers" class="w-3.5 h-3.5" />
                                    </span>
                                    <div class="min-w-0">
                                        <p class="font-bold text-green-900 leading-none truncate">{{ user.user_nickname
                                            }}</p>
                                        <p class="mono text-[10px] text-slate-400 mt-1 truncate">ID #{{ user.user_id }}
                                        </p>
                                    </div>
                                </div>
                            </td>
                            <td class="py-3 px-3">
                                <p class="text-[13px] font-bold text-green-900 truncate">{{ user.user_fname }} {{
                                    user.user_lname }}</p>
                                <p class="mono text-[10px] text-emerald-700/50 mt-1 truncate">{{ user.user_nickname ===
                                    auth.user?.nickname ? 'this account' : 'staff account' }}</p>
                            </td>
                            <td class="py-3 px-3">
                                <button @click="openPermissionsModal(user)" class="text-left group block w-full">
                                    <span class="inline-flex px-2.5 py-1 rounded-full border mono text-[11px] font-bold"
                                        :class="getPermCount(user) === ALL_PAGES.length ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white border-emerald-200 text-emerald-700'">
                                        {{ getPermCount(user) }} / {{ ALL_PAGES.length }} pages
                                    </span>
                                    <div v-if="permSummary(user).length"
                                        class="mono text-[10px] text-emerald-700/50 mt-1 leading-tight truncate">
                                        {{ permSummary(user).join(' · ') }}
                                    </div>
                                    <div v-else class="mono text-[10px] text-gray-400 mt-1">no page access</div>
                                </button>
                            </td>
                            <td class="py-3 px-3 text-right whitespace-nowrap">
                                <div class="inline-flex items-center gap-1">
                                    <button @click="openPermissionsModal(user)"
                                        class="w-8 h-8 rounded-xl bg-white border-2 border-emerald-100 text-emerald-600 hover:bg-emerald-600 hover:text-white hover:border-emerald-600 flex items-center justify-center transition shadow-sm"
                                        :title="$t('user.permissionsTitle')">
                                        <FontAwesome :icon="faShieldAlt" class="w-3 h-3" />
                                    </button>
                                    <button v-if="canEdit" @click="openEditModal(user)"
                                        class="w-8 h-8 rounded-xl bg-white border-2 border-emerald-100 text-emerald-600 hover:bg-emerald-600 hover:text-white hover:border-emerald-600 flex items-center justify-center transition shadow-sm"
                                        :title="$t('common.edit')">
                                        <FontAwesome :icon="faEdit" class="w-3 h-3" />
                                    </button>
                                    <button v-if="canDelete && user.user_nickname !== auth.user?.nickname"
                                        @click="deleteUser(user)"
                                        class="w-8 h-8 rounded-xl bg-white border-2 border-red-100 text-red-500 hover:bg-red-500 hover:text-white hover:border-red-500 flex items-center justify-center transition shadow-sm"
                                        :title="$t('common.delete')">
                                        <FontAwesome :icon="faTrash" class="w-3 h-3" />
                                    </button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                    <tbody v-else>
                        <tr>
                            <td colspan="4" class="py-14 text-center">
                                <div class="flex flex-col items-center gap-3 px-6">
                                    <div
                                        class="w-12 h-12 rounded-2xl bg-emerald-50 border border-emerald-100 flex items-center justify-center text-emerald-600">
                                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none"
                                            stroke="currentColor" stroke-width="1.8">
                                            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
                                            <circle cx="9" cy="7" r="4" />
                                            <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
                                        </svg>
                                    </div>
                                    <p class="mono text-xs font-bold tracking-[0.14em] text-emerald-700">{{
                                        $t('user.noUsers') }}</p>
                                    <p class="text-sm text-slate-500 max-w-md">{{ $t('user.addHint') }}</p>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <!-- Mobile cards -->
            <div v-if="!loading && users.length > 0"
                class="md:hidden p-2.5 space-y-2.5 max-h-[52vh] overflow-auto bg-emerald-50/20">
                <div v-for="user in users" :key="`m-${user.user_id}`"
                    class="rounded-2xl border border-emerald-100 bg-white p-3.5 shadow-sm">
                    <div class="flex items-start justify-between gap-2">
                        <div class="flex items-center gap-2.5 min-w-0">
                            <span
                                class="w-9 h-9 rounded-xl bg-emerald-600 text-white flex items-center justify-center shrink-0">
                                <FontAwesome :icon="faUsers" class="w-3.5 h-3.5" />
                            </span>
                            <div class="min-w-0">
                                <p class="font-bold text-sm text-green-900 truncate">{{ user.user_nickname }}</p>
                                <p class="mono text-[10px] text-slate-400 mt-1">#{{ user.user_id }} • {{ user.user_fname
                                    }} {{ user.user_lname }}</p>
                            </div>
                        </div>
                        <span class="shrink-0 inline-flex px-2 py-1 rounded-full border mono text-[10px] font-bold"
                            :class="getPermCount(user) === ALL_PAGES.length ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white border-emerald-200 text-emerald-700'">
                            {{ getPermCount(user) }}/{{ ALL_PAGES.length }}
                        </span>
                    </div>
                    <p v-if="permSummary(user).length" class="mt-2.5 mono text-[10px] text-emerald-700/50 truncate">{{
                        permSummary(user).join(' · ') }}</p>
                    <div class="mt-3 flex gap-2">
                        <button @click="openPermissionsModal(user)"
                            class="flex-1 py-2.5 rounded-xl bg-white border-2 border-emerald-100 text-emerald-600 mono text-xs font-bold">{{
                            $t('user.colAccess') }}</button>
                        <button v-if="canEdit" @click="openEditModal(user)"
                            class="flex-1 py-2.5 rounded-xl bg-emerald-600 text-white mono text-xs font-bold shadow-sm">{{
                            $t('common.edit') }}</button>
                        <button v-if="canDelete && user.user_nickname !== auth.user?.nickname" @click="deleteUser(user)"
                            class="flex-1 py-2.5 rounded-xl bg-white border-2 border-red-100 text-red-600 mono text-xs font-bold">{{
                            $t('common.delete') }}</button>
                    </div>
                </div>
            </div>
            <div v-else-if="!loading" class="md:hidden p-10 text-center">
                <p class="mono text-xs font-bold tracking-[0.14em] text-emerald-700">{{ $t('user.noUsers') }}</p>
                <p class="text-sm text-slate-500 mt-1">{{ $t('user.addHint') }}</p>
            </div>

            <div
                class="px-4 py-2.5 bg-white/90 backdrop-blur border-t border-emerald-100 flex items-center justify-between mono text-[10px]">
                <span class="text-emerald-700/60">{{ users.length }} of {{ users.length }} users</span>
                <span class="hidden sm:inline text-slate-400">{{ $t('user.footerHint') }}</span>
            </div>
        </div>

        <!-- Add/Edit Modal -->
        <div v-if="showModal" class="fixed inset-0 z-[100] flex items-center justify-center p-4">
            <div class="absolute inset-0 bg-black/40 backdrop-blur-[2px]" @click="showModal = false"></div>
            <div
                class="relative w-full max-w-md rounded-2xl bg-white border border-emerald-100 shadow-2xl p-6 space-y-4">
                <h3 class="display text-lg font-extrabold text-green-900">{{ modalTitle }}</h3>

                <div class="space-y-3">
                    <div>
                        <label class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">{{
                            $t('user.colNickname') }}</label>
                        <input v-model="form.nickname" type="text"
                            class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 transition mt-1"
                            :placeholder="$t('user.phNickname')">
                    </div>

                    <div>
                        <label class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">
                            {{ $t('login.password') }} <span v-if="modalMode === 'edit'" class="text-emerald-500">(leave
                                blank to keep)</span>
                        </label>
                        <input v-model="form.password" type="password"
                            class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 transition mt-1"
                            placeholder="••••••••">
                    </div>

                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">{{
                                $t('employee.firstName') }}</label>
                            <input v-model="form.fname" type="text"
                                class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 transition mt-1"
                                :placeholder="$t('user.phFirst')">
                        </div>
                        <div>
                            <label class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">{{
                                $t('employee.lastName') }}</label>
                            <input v-model="form.lname" type="text"
                                class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 transition mt-1"
                                :placeholder="$t('user.phLast')">
                        </div>
                    </div>
                </div>

                <div class="flex items-center gap-3 pt-2">
                    <button @click="saveUser" :disabled="saving"
                        class="btn-emerald px-6 h-11 rounded-xl text-sm font-bold inline-flex items-center gap-2 disabled:opacity-40 transition">
                        <span v-if="saving"
                            class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                        {{ saving ? 'Saving…' : $t('common.save') }}
                    </button>
                    <button @click="showModal = false"
                        class="px-6 h-11 rounded-xl text-sm font-bold text-emerald-700 hover:bg-emerald-50 transition">{{
                            $t('common.cancel') }}</button>
                </div>
            </div>
        </div>

        <!-- Permissions Modal -->
        <div v-if="showPermissionsModal" class="fixed inset-0 z-[100] flex items-center justify-center p-4">
            <div class="absolute inset-0 bg-black/40 backdrop-blur-[2px]" @click="showPermissionsModal = false"></div>
            <div
                class="relative w-full max-w-lg rounded-2xl bg-white border border-emerald-100 shadow-2xl p-6 space-y-4">
                <div class="flex items-center gap-3">
                    <div
                        class="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-green-600 flex items-center justify-center shrink-0">
                        <font-awesome :icon="faShieldAlt" class="w-5 h-5 text-white" />
                    </div>
                    <div>
                        <h3 class="display text-lg font-extrabold text-green-900">{{ $t('user.pagePermissions') }}</h3>
                        <p class="text-xs text-emerald-600">{{ permissionsUser?.user_nickname }}</p>
                    </div>
                </div>

                <p class="mono text-[10px] tracking-[0.12em] text-emerald-700/50">
                    V = VIEW &nbsp; A = ADD &nbsp; E = EDIT &nbsp; D = DELETE
                </p>

                <!-- One row per page, four independent action toggles. View is the
                     master switch: you cannot write to a page you cannot open. -->
                <div class="space-y-2 max-h-[45vh] overflow-y-auto pr-1">
                    <div v-for="page in ALL_PAGES" :key="page.key"
                        class="rounded-xl border border-emerald-100 px-4 py-3" :class="[
                            editingPermissions[page.key].can_view ? 'bg-emerald-50/40' : 'bg-white',
                            editingSelf && page.key === 'users' && selfLockoutBlocked ? '!border-red-300 ring-1 ring-red-200' : ''
                        ]">
                        <div class="flex items-center justify-between gap-3">
                            <span class="text-sm font-medium text-green-900">{{ page.label }}</span>
                            <div class="flex items-center gap-1.5">
                                <button @click="toggleAll(page.key)" :title="$t('user.toggleAll')"
                                    class="px-2 h-7 rounded-lg text-[10px] font-bold tracking-wider transition" :class="ACTIONS.every(a => editingPermissions[page.key][a])
                                        ? 'bg-emerald-500 text-white'
                                        : 'bg-emerald-50 text-emerald-700 hover:bg-emerald-100'">
                                    {{ $t('common.all') }}
                                </button>
                                <button v-for="(action, i) in ACTIONS" :key="action"
                                    @click="action === 'can_view' ? toggleView(page.key) : toggleAction(page.key, action)"
                                    :title="ACTION_LABELS[action]"
                                    class="w-7 h-7 rounded-lg text-[10px] font-bold transition" :class="editingPermissions[page.key][action]
                                        ? 'bg-emerald-500 text-white'
                                        : 'bg-gray-100 text-gray-400 hover:bg-gray-200'">
                                    {{ ['V', 'A', 'E', 'D'][i] }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <div v-if="selfLockoutBlocked"
                    class="rounded-xl bg-red-50 border border-red-200 px-4 py-3 text-xs text-red-700">
                    {{ selfLockoutReason }}
                </div>

                <div class="flex items-center gap-3 pt-2">
                    <button @click="savePermissions" :disabled="permissionsSaving || selfLockoutBlocked"
                        class="btn-emerald px-6 h-11 rounded-xl text-sm font-bold inline-flex items-center gap-2 disabled:opacity-40 transition">
                        <span v-if="permissionsSaving"
                            class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                        {{ permissionsSaving ? 'Saving…' : $t('common.save') }}
                    </button>
                    <button @click="showPermissionsModal = false"
                        class="px-6 h-11 rounded-xl text-sm font-bold text-emerald-700 hover:bg-emerald-50 transition">{{
                            $t('common.cancel') }}</button>
                </div>
            </div>
        </div>
    </div>
</template>
