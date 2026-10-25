<script setup>
import { useAuthStore } from '~/store/auth';
import {
  faGaugeHigh, faChartColumn, faUserGroup, faFingerprint, faClock,
  faPlaneDeparture, faReceipt, faLocationDot,
  faSliders, faArrowRightFromBracket, faXmark,
  faChevronLeft, faChevronRight, faChevronDown, faUsers, faGlobe, faCheck,
} from '@fortawesome/free-solid-svg-icons'
import { SMData } from '~/components/SimpleModal.vue';

const route = useRoute();
// Locale switcher: the list of locales comes from the i18n module, so adding
// a language to nuxt.config.ts automatically adds it to the dropdown.
const { locale, locales, setLocale } = useI18n();
const showLocaleMenu = ref(false);
const localeMenuRef = ref(null);
const localeLabel = computed(() => locales.value.find(l => l.code === locale.value)?.name || locale.value);
function toggleLocaleMenu() { showLocaleMenu.value = !showLocaleMenu.value; if (showLocaleMenu.value) showUserMenu.value = false; }
function closeLocaleMenu() { showLocaleMenu.value = false }
async function pickLocale(code) {
  closeLocaleMenu();
  setLocale(code);
  try { localStorage.setItem('lulu:locale', code); } catch { }
}
onMounted(() => {
  try {
    const saved = localStorage.getItem('lulu:locale');
    if (saved && locales.value.some(l => l.code === saved) && saved !== locale.value) setLocale(saved);
  } catch { }
});
const openMenu = ref(false);
const showUserMenu = ref(false);
const userMenuRef = ref(null);
const sm_data = new SMData();
const auth = useAuthStore();

function toggleMenu() { openMenu.value = !openMenu.value; if (openMenu.value) showUserMenu.value = false; }
function closeMenu() { openMenu.value = false }
function toggleUserMenu() { showUserMenu.value = !showUserMenu.value; if (showUserMenu.value) closeLocaleMenu(); }
function closeUserMenu() { showUserMenu.value = false }
function go(path) { closeMenu(); closeUserMenu(); navigateTo(path, { replace: true }) }
watch(() => route.path, () => { closeMenu(); closeUserMenu(); closeLocaleMenu(); });
function onKey(e) {
  if (e.key === 'Escape') { closeMenu(); closeUserMenu(); closeLocaleMenu(); }
  const tag = (e.target && e.target.tagName) || '';
  if (e.key === '[' && !/^(INPUT|TEXTAREA|SELECT)$/.test(tag)) { e.preventDefault(); toggleSidebar(); }
}
function onDocClick(e) {
  if (showUserMenu.value) {
    const el = userMenuRef.value;
    if (el && !el.contains(e.target)) closeUserMenu();
  }
  if (showLocaleMenu.value) {
    const el = localeMenuRef.value;
    if (el && !el.contains(e.target)) closeLocaleMenu();
  }
}
onMounted(() => {
  window.addEventListener('keydown', onKey);
  document.addEventListener('click', onDocClick);
  try { const v = localStorage.getItem('lulu:sidebar-compact'); if (v !== null) sidebarCompact.value = v === '1'; } catch { }
});
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey);
  document.removeEventListener('click', onDocClick);
});

// --- Dual-mode sidebar: expanded (248px) vs compact (72px) ---
const sidebarCompact = ref(false);
function toggleSidebar() { sidebarCompact.value = !sidebarCompact.value; try { localStorage.setItem('lulu:sidebar-compact', sidebarCompact.value ? '1' : '0'); } catch { } }
watch(sidebarCompact, (v) => { try { localStorage.setItem('lulu:sidebar-compact', v ? '1' : '0'); } catch { } });

const NAV = [
  { path: '/', label: 'nav.dashboard', icon: faGaugeHigh, pageKey: 'dashboard' },
  { path: '/employee', label: 'nav.employees', icon: faUserGroup, pageKey: 'employee' },
  { path: '/shift', label: 'nav.shifts', icon: faClock, pageKey: 'shift' },
  { path: '/report', label: 'nav.reports', icon: faChartColumn, pageKey: 'report' },
  { path: '/controller', label: 'nav.devices', icon: faFingerprint, pageKey: 'controller' },
  { path: '/leave', label: 'nav.leave', icon: faPlaneDeparture, pageKey: 'leave' },
  { path: '/field', label: 'nav.field', icon: faLocationDot, pageKey: 'field' },
  { path: '/payroll', label: 'nav.payroll', icon: faReceipt, pageKey: 'payroll' },
]
const isActive = (p) => route.path === p || (p !== '/' && route.path.startsWith(p + '/'))
const isActiveOn = isActive

// Filter nav items based on user permissions
const visibleNav = computed(() => NAV.filter(n => auth.hasPageAccess(n.pageKey)))

const initials = computed(() => auth.initials)
const userName = computed(() => auth.displayName)
const { t } = useI18n()
const pageTitle = computed(() => {
  const hit = NAV.find(n => n.path === route.path)
  if (hit) return t(hit.label)
  return route.path.startsWith('/settings') ? t('nav.settings') : t('nav.pageTitleFallback')
})

function actCancel() { sm_data.clear() }
function actOk() { sm_data.clear(); auth.logout() }
function modalLogout() {
  closeUserMenu();
  sm_data.setText(t("nav.logOut"), t("login.logOutConfirm"))
  sm_data.showOKCancel()
}
</script>

<template>
  <div class="min-h-screen lg:h-screen flex bg-canvas overflow-x-hidden lg:overflow-hidden">
    <a href="#main" class="skip-link">{{ $t('common.skipToContent') }}</a>

    <!-- ============ SIDEBAR: 264px expanded, 76px rail ============ -->
    <aside :style="{ width: sidebarCompact ? '76px' : '264px' }"
      class="hidden lg:flex shrink-0 flex-col relative glass-dark sticky top-0 h-screen z-40 transition-[width] duration-300 ease-out">

      <!-- Brand + rail toggle (chevron appears on hover, stays put in compact) -->
      <div class="relative border-b border-white/10 shrink-0">
        <div class="group flex items-center h-[72px]" :class="sidebarCompact ? 'justify-center px-2' : 'gap-3 px-4'">
          <div class="w-[38px] h-[38px] rounded-xl overflow-hidden ring-1 ring-white/15 shrink-0">
            <img src="/lulu-192.png" alt="LuLu Attendance" class="w-full h-full object-cover">
          </div>

          <div v-show="!sidebarCompact" class="min-w-0 leading-tight flex-1">
            <p class="display text-[15px] text-white truncate">LuLu</p>
            <p class="mono text-[9.5px] tracking-[0.18em] text-emerald-200/50 truncate">ATTENDANCE</p>
          </div>

          <button v-if="!sidebarCompact" type="button" @click="toggleSidebar"
            :aria-expanded="sidebarCompact ? 'false' : 'true'" aria-label="Collapse sidebar"
            title="Collapse sidebar ( [ )"
            class="w-8 h-8 shrink-0 grid place-content-center rounded-lg text-emerald-200/70 opacity-0 group-hover:opacity-100 focus-visible:opacity-100 hover:bg-white/10 hover:text-white transition">
            <font-awesome :icon="faChevronLeft" class="w-3 h-3" />
          </button>
        </div>

        <div v-if="sidebarCompact" class="pb-2 flex justify-center">
          <button type="button" @click="toggleSidebar" :aria-expanded="sidebarCompact ? 'false' : 'true'"
            aria-label="Expand sidebar" title="Expand sidebar ( [ )"
            class="w-9 h-9 grid place-content-center rounded-lg text-emerald-200/70 hover:bg-white/10 hover:text-white transition">
            <font-awesome :icon="faChevronRight" class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <!-- Nav -->
      <nav class="relative flex-1 overflow-y-auto overflow-x-hidden py-3 min-w-0"
        :class="sidebarCompact ? 'px-2 space-y-1' : 'px-3 space-y-1'" aria-label="Main">
        <p v-show="!sidebarCompact"
          class="mono text-[9.5px] font-bold tracking-[0.2em] text-emerald-200/40 px-3 pb-1.5">{{ $t('nav.menu') }}</p>

        <template v-for="n in visibleNav" :key="n.path">
          <a :href="n.path" @click.prevent="go(n.path)" :aria-current="isActive(n.path) ? 'page' : undefined"
            :title="sidebarCompact ? $t(n.label) : undefined"
            class="group relative flex items-center rounded-xl text-[13.5px] font-semibold transition-colors duration-150 cursor-pointer min-w-0 min-h-[44px]"
            :class="[
              sidebarCompact ? 'justify-center px-2' : 'gap-3 px-3',
              isActive(n.path)
                ? 'nav-active'
                : 'text-emerald-50/75 hover:text-white hover:bg-white/[0.08]'
            ]">
            <font-awesome :icon="n.icon" class="text-[15px] w-5 text-center shrink-0 opacity-90" />
            <span v-show="!sidebarCompact" class="truncate">{{ $t(n.label) }}</span>
            <span v-if="sidebarCompact"
              class="pointer-events-none hidden group-hover:block absolute left-[calc(100%+12px)] top-1/2 -translate-y-1/2 whitespace-nowrap px-2.5 py-1.5 rounded-lg bg-green-950 text-white text-xs font-semibold shadow-tint-lg z-50">
              {{ $t(n.label) }}
            </span>
          </a>
        </template>

        <div class="my-3 h-px bg-white/10"></div>

        <a v-if="auth.hasPageAccess('users')" href="/users" @click.prevent="go('/users')"
          :title="sidebarCompact ? $t('nav.users') : undefined"
          class="group relative flex items-center rounded-xl text-[13.5px] font-semibold transition-colors duration-150 cursor-pointer min-w-0 min-h-[44px]"
          :class="[sidebarCompact ? 'justify-center px-2' : 'gap-3 px-3', isActive('/users') ? 'nav-active' : 'text-emerald-50/75 hover:text-white hover:bg-white/[0.08]']">
          <font-awesome :icon="faUsers" class="text-[15px] w-5 text-center shrink-0 opacity-90" />
          <span v-show="!sidebarCompact">{{ $t('nav.users') }}</span>
          <span v-if="sidebarCompact"
            class="pointer-events-none hidden group-hover:block absolute left-[calc(100%+12px)] top-1/2 -translate-y-1/2 whitespace-nowrap px-2.5 py-1.5 rounded-lg bg-green-950 text-white text-xs font-semibold shadow-tint-lg z-50">
            {{ $t('nav.users') }}
          </span>
        </a>

        <a v-if="auth.hasPageAccess('settings')" href="/settings" @click.prevent="go('/settings')"
          :title="sidebarCompact ? $t('nav.settings') : undefined"
          class="group relative flex items-center rounded-xl text-[13.5px] font-semibold transition-colors duration-150 cursor-pointer min-w-0 min-h-[44px]"
          :class="[sidebarCompact ? 'justify-center px-2' : 'gap-3 px-3', isActive('/settings') ? 'nav-active' : 'text-emerald-50/75 hover:text-white hover:bg-white/[0.08]']">
          <font-awesome :icon="faSliders" class="text-[15px] w-5 text-center shrink-0 opacity-90" />
          <span v-show="!sidebarCompact">{{ $t('nav.settings') }}</span>
          <span v-if="sidebarCompact"
            class="pointer-events-none hidden group-hover:block absolute left-[calc(100%+12px)] top-1/2 -translate-y-1/2 whitespace-nowrap px-2.5 py-1.5 rounded-lg bg-green-950 text-white text-xs font-semibold shadow-tint-lg z-50">
            {{ $t('nav.settings') }}
          </span>
        </a>
      </nav>

    </aside>

    <!-- ============ MAIN ============ -->
    <div class="flex-1 flex flex-col min-w-0 min-h-0 lg:h-screen lg:overflow-hidden">
      <header class="shrink-0 sticky top-0 z-30 glass border-b border-border">
        <div class="h-[2px] w-full bg-gradient-to-r from-emerald-500 via-lime-400 to-emerald-600 opacity-50"></div>
        <div class="flex h-16 items-center gap-3 px-4 sm:px-5">
          <button type="button" @click="toggleMenu" :aria-expanded="openMenu ? 'true' : 'false'"
            aria-controls="mobile-nav" aria-label="Toggle navigation"
            class="lg:hidden w-11 h-11 grid place-content-center rounded-xl overflow-hidden ring-1 ring-border-strong hover:ring-emerald-300 active:scale-95 transition shrink-0">
            <img src="/lulu-192.png" alt="LuLu Attendance" class="w-full h-full object-cover">
          </button>

          <div class="min-w-0">
            <h1 class="display text-[17px] text-text truncate">{{ pageTitle }}</h1>
            <p class="hidden sm:block mono text-[9.5px] tracking-[0.16em] text-text-muted">LULU {{
              $t('brand.systemName') }}
            </p>
          </div>

          <div class="ml-auto flex items-center gap-1.5 sm:gap-2 min-w-0">
            <NotifyBell />

            <div ref="localeMenuRef" class="relative shrink-0">
              <button type="button" @click.stop="toggleLocaleMenu" :aria-expanded="showLocaleMenu ? 'true' : 'false'"
                aria-haspopup="menu" :aria-label="localeLabel"
                class="flex items-center gap-1.5 rounded-xl px-2.5 min-h-11 text-primary-deep hover:bg-surface-raised active:bg-surface-sunken transition">
                <font-awesome :icon="faGlobe" class="w-4 h-4" />
                <span class="mono text-[11px] font-bold tracking-wider uppercase">{{ locale }}</span>
                <font-awesome :icon="faChevronDown" class="w-2.5 h-2.5 transition"
                  :class="showLocaleMenu ? 'rotate-180' : ''" />
              </button>
              <div v-if="showLocaleMenu"
                class="absolute right-0 top-full mt-2 min-w-[11rem] rounded-xl bg-surface border border-border shadow-tint-lg overflow-hidden z-50 py-1"
                role="menu">
                <button v-for="l in locales" :key="l.code" type="button" role="menuitem"
                  @click.stop="pickLocale(l.code)"
                  class="flex w-full items-center justify-between gap-3 px-4 min-h-11 text-sm transition text-left"
                  :class="l.code === locale ? 'bg-surface-raised font-semibold text-text' : 'text-text-muted hover:bg-surface-raised/60'">
                  <span class="truncate">{{ l.name }}</span>
                  <font-awesome v-if="l.code === locale" :icon="faCheck" class="w-3.5 h-3.5 text-primary shrink-0" />
                </button>
              </div>
            </div>

            <div ref="userMenuRef" class="relative flex items-center min-w-0">
              <button type="button" @click.stop="toggleUserMenu" :aria-expanded="showUserMenu ? 'true' : 'false'"
                aria-haspopup="menu" aria-label="User menu"
                class="flex items-center gap-2 rounded-xl px-1.5 min-h-11 hover:bg-surface-raised active:bg-surface-sunken transition min-w-0">
                <div
                  class="w-8 h-8 rounded-lg bg-gradient-to-br from-emerald-500 to-green-700 grid place-content-center text-white text-[11px] font-bold shrink-0">
                  {{ initials }}</div>
                <div class="hidden lg:block text-left leading-tight min-w-0">
                  <p class="text-[12.5px] font-semibold text-text truncate max-w-[9rem]">{{ userName }}</p>
                </div>
                <font-awesome :icon="faChevronDown" class="hidden sm:block w-3 h-3 text-text-muted transition"
                  :class="showUserMenu ? 'rotate-180' : ''" />
              </button>

              <div v-if="showUserMenu"
                class="absolute right-0 top-full mt-2 w-56 rounded-xl bg-surface border border-border shadow-tint-lg overflow-hidden z-50"
                role="menu" aria-label="User menu">
                <div class="px-4 py-3 bg-surface-raised/60 border-b border-border">
                  <p class="text-sm font-semibold text-text truncate">{{ userName }}</p>
                </div>
                <button @click.stop="modalLogout" type="button" role="menuitem"
                  class="flex w-full items-center gap-3 px-4 min-h-11 text-sm font-medium text-text hover:bg-red-50 hover:text-danger transition text-left">
                  <font-awesome :icon="faArrowRightFromBracket" class="w-4 h-4 shrink-0" />
                  <span>{{ $t('nav.logOut') }}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </header>

      <!-- Mobile nav sheet -->
      <div v-if="openMenu" id="mobile-nav" class="lg:hidden fixed inset-0 z-[80]" role="dialog" aria-modal="true">
        <div class="absolute inset-0 bg-green-950/50 backdrop-blur-[2px]" @click="closeMenu"></div>
        <nav class="absolute start-0 top-0 bottom-0 w-[280px] max-w-[82vw] glass-dark flex flex-col shadow-tint-lg">
          <div class="flex items-center gap-3 h-[72px] px-4 border-b border-white/10 shrink-0">
            <div class="w-9 h-9 rounded-xl overflow-hidden ring-1 ring-white/15 shrink-0">
              <img src="/lulu-192.png" alt="LuLu Attendance" class="w-full h-full object-cover">
            </div>
            <div class="min-w-0 leading-tight">
              <p class="display text-sm text-white">LuLu</p>
              <p class="mono text-[9.5px] tracking-[0.18em] text-emerald-200/50">ATTENDANCE</p>
            </div>
            <button type="button" @click="closeMenu" aria-label="Close navigation"
              class="ms-auto w-11 h-11 grid place-content-center rounded-xl text-emerald-100 hover:text-white hover:bg-white/10 transition shrink-0">
              <font-awesome :icon="faXmark" class="w-4 h-4" />
            </button>
          </div>

          <div class="flex-1 overflow-y-auto overflow-x-hidden px-3 py-3 space-y-1">
            <p class="mono text-[9.5px] font-bold tracking-[0.2em] text-emerald-200/40 px-3 pb-1.5">{{ $t('nav.menu') }}
            </p>
            <a v-for="n in visibleNav" :key="n.path" :href="n.path" @click.prevent="go(n.path)"
              class="flex items-center gap-3 px-3 min-h-[44px] rounded-xl font-semibold text-[13.5px] transition cursor-pointer"
              :class="isActive(n.path) ? 'nav-active' : 'text-emerald-50/80 hover:bg-white/[0.08]'">
              <font-awesome :icon="n.icon" class="w-4 h-4 shrink-0" />
              <span class="truncate">{{ $t(n.label) }}</span>
            </a>
            <div class="my-3 h-px bg-white/10"></div>
            <a v-if="auth.hasPageAccess('users')" href="/users" @click.prevent="go('/users')"
              class="flex items-center gap-3 px-3 min-h-[44px] rounded-xl font-semibold text-[13.5px] transition cursor-pointer"
              :class="isActive('/users') ? 'nav-active' : 'text-emerald-50/80 hover:bg-white/[0.08]'">
              <font-awesome :icon="faUsers" class="w-4 h-4 shrink-0" />
              <span>{{ $t('nav.users') }}</span>
            </a>
            <a v-if="auth.hasPageAccess('settings')" href="/settings" @click.prevent="go('/settings')"
              class="flex items-center gap-3 px-3 min-h-[44px] rounded-xl font-semibold text-[13.5px] transition cursor-pointer"
              :class="isActive('/settings') ? 'nav-active' : 'text-emerald-50/80 hover:bg-white/[0.08]'">
              <font-awesome :icon="faSliders" class="w-4 h-4 shrink-0" />
              <span>{{ $t('nav.settings') }}</span>
            </a>
          </div>

          <div class="border-t border-white/10 p-3 pb-[max(0.75rem,env(safe-area-inset-bottom))] shrink-0">
            <button @click="closeMenu(); modalLogout()" type="button"
              class="flex w-full items-center gap-3 px-3 min-h-[44px] rounded-xl font-semibold text-sm text-red-300 hover:bg-red-500/10 transition cursor-pointer">
              <font-awesome :icon="faArrowRightFromBracket" class="w-4 h-4 shrink-0" />
              <span>{{ $t('nav.logOut') }}</span>
            </button>
          </div>
        </nav>
      </div>

      <main id="main" class="flex-1 min-h-0 p-3 sm:p-4 lg:p-5 lg:overflow-y-auto lg:overflow-x-hidden flex flex-col">
        <slot />
      </main>
    </div>
  </div>

  <SimpleModal :options="sm_data.get()" v-on:cancel="actCancel" v-on:ok="actOk" v-show="sm_data.isShow()" />
  <ToastStack />
</template>
