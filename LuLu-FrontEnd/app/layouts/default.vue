<script setup>
import { useAuthStore } from '~/store/auth';
import {
  faGaugeHigh, faChartColumn, faUserGroup, faFingerprint, faClock,
  faSliders, faArrowRightFromBracket, faXmark,
  faChevronLeft, faChevronRight, faChevronDown,
} from '@fortawesome/free-solid-svg-icons'
import { SMData } from '~/components/SimpleModal.vue';

const route = useRoute();
const openMenu = ref(false);
const showUserMenu = ref(false);
const userMenuRef = ref(null);
const sm_data = new SMData();
const auth = useAuthStore();

function toggleMenu() { openMenu.value = !openMenu.value; if (openMenu.value) showUserMenu.value = false; }
function closeMenu() { openMenu.value = false }
function toggleUserMenu() { showUserMenu.value = !showUserMenu.value }
function closeUserMenu() { showUserMenu.value = false }
function go(path) { closeMenu(); closeUserMenu(); navigateTo(path, { replace: true }) }
watch(() => route.path, () => { closeMenu(); closeUserMenu(); });
function onKey(e) { if (e.key === 'Escape') { closeMenu(); closeUserMenu(); } }
function onDocClick(e) {
  if (!showUserMenu.value) return;
  const el = userMenuRef.value;
  if (el && !el.contains(e.target)) closeUserMenu();
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

// ——— Dual-mode sidebar: expanded (248px) vs compact (72px) ———
const sidebarCompact = ref(false);
function toggleSidebar() { sidebarCompact.value = !sidebarCompact.value; try { localStorage.setItem('lulu:sidebar-compact', sidebarCompact.value ? '1' : '0'); } catch { } }
watch(sidebarCompact, (v) => { try { localStorage.setItem('lulu:sidebar-compact', v ? '1' : '0'); } catch { } });

const NAV = [
  { path: '/', label: 'Dashboard', icon: faGaugeHigh },
  { path: '/employee', label: 'Employees', icon: faUserGroup },
  { path: '/shift', label: 'Shifts', icon: faClock },
  { path: '/report', label: 'Reports', icon: faChartColumn },
  { path: '/controller', label: 'Devices', icon: faFingerprint },
]
const isActive = (p) => route.path === p || (p !== '/' && route.path.startsWith(p + '/'))
const isActiveOn = isActive

const initials = computed(() => auth.initials)
const userName = computed(() => auth.displayName)
const pageTitle = computed(() => NAV.find(n => n.path === route.path)?.label
  || (route.path.startsWith('/settings') ? 'Settings' : 'LuLu Attendance'))

function actCancel() { sm_data.clear() }
function actOk() { sm_data.clear(); auth.logout() }
function modalLogout() {
  closeUserMenu();
  sm_data.setText("Log out", "Log out of LuLu Attendance?")
  sm_data.showOKCancel()
}
</script>

<template>
  <div
    class="min-h-screen lg:h-screen flex bg-[#f6fdf7] tech-pattern tech-grid-soft overflow-x-hidden lg:overflow-hidden">
    <!-- ============ SIDEBAR (desktop: always visible, 2 widths) ============ -->
    <aside :style="{ width: sidebarCompact ? '72px' : '248px' }"
      class="hidden lg:flex shrink-0 flex-col relative glass-dark shadow-[4px_0_24px_rgba(5,46,22,.18)] z-50 sticky top-0 h-screen transition-all duration-300 ease-in-out overflow-visible">
      <div class="pointer-events-none absolute inset-0 opacity-[0.06] tech-dots"></div>
      <!-- Brand -->
      <div class="relative flex items-center h-[68px] border-b border-white/10 shrink-0"
        :class="sidebarCompact ? 'justify-center px-2' : 'gap-3 px-4'">
        <div class="w-10 h-10 rounded-xl overflow-hidden shadow-lg shadow-emerald-900/40 ring-1 ring-white/20 shrink-0">
          <img src="/lulu-192.png" alt="LuLu Attendance" class="w-full h-full object-cover">
        </div>
        <div v-show="!sidebarCompact" class="min-w-0 leading-tight">
          <p class="display text-sm font-extrabold tracking-tight text-white truncate">LuLu</p>
          <p class="mono text-[10px] tracking-[0.14em] text-emerald-200/60 truncate">ATTENDANCE</p>
        </div>
      </div>

      <!-- Nav — x-hidden prevents horizontal scroll in compact -->
      <nav class="relative flex-1 overflow-y-auto overflow-x-hidden py-4 space-y-1 min-w-0"
        :class="sidebarCompact ? 'px-2' : 'px-3'" aria-label="Main">
        <p v-show="!sidebarCompact" class="mono text-[10px] font-bold tracking-[0.18em] text-emerald-200/40 px-3 pb-2">
          MENU</p>
        <template v-for="n in NAV" :key="n.path">
          <a :href="n.path" @click.prevent="go(n.path)" :aria-current="isActive(n.path) ? 'page' : undefined"
            :title="sidebarCompact ? n.label : undefined"
            class="group flex items-center rounded-xl text-sm font-semibold transition-all duration-150 cursor-pointer relative min-w-0"
            :class="[
              sidebarCompact ? 'justify-center px-2 py-2.5' : 'gap-3 px-3 py-2.5',
              isActive(n.path)
                ? 'bg-gradient-to-r from-emerald-500 to-emerald-600 text-white shadow-[0_4px_14px_-4px_rgba(16,185,129,.6)]'
                : 'text-emerald-50/90 hover:text-white hover:bg-white/10'
            ]">
            <font-awesome :icon="n.icon" class="text-[15px] w-5 text-center shrink-0" />
            <span v-show="!sidebarCompact" class="truncate">{{ n.label }}</span>
            <span v-if="sidebarCompact"
              class="pointer-events-none hidden group-hover:block absolute left-[calc(100%+10px)] top-1/2 -translate-y-1/2 whitespace-nowrap px-2.5 py-1.5 rounded-lg bg-green-950 text-white text-xs font-semibold shadow-xl z-50">
              {{ n.label }}
            </span>
          </a>
        </template>

        <div class="my-4 h-px bg-white/10"></div>
        <a href="/settings" @click.prevent="go('/settings')" :title="sidebarCompact ? 'Settings' : undefined"
          class="group flex items-center rounded-xl text-sm font-semibold transition-all duration-150 cursor-pointer relative min-w-0"
          :class="[
            sidebarCompact ? 'justify-center px-2 py-2.5' : 'gap-3 px-3 py-2.5',
            isActive('/settings') ? 'bg-white/15 text-white' : 'text-emerald-50/90 hover:text-white hover:bg-white/10'
          ]">
          <font-awesome :icon="faSliders" class="text-[15px] w-5 text-center shrink-0" />
          <span v-show="!sidebarCompact">Settings</span>
          <span v-if="sidebarCompact"
            class="pointer-events-none hidden group-hover:block absolute left-[calc(100%+10px)] top-1/2 -translate-y-1/2 whitespace-nowrap px-2.5 py-1.5 rounded-lg bg-green-950 text-white text-xs font-semibold shadow-xl z-50">
            Settings
          </span>
        </a>
      </nav>

      <!-- Middle toggle — floating di tengah seam (pattern: pill chevron centered on edge) -->
      <button type="button" @click="toggleSidebar" :aria-label="sidebarCompact ? 'Expand sidebar' : 'Collapse sidebar'"
        :title="sidebarCompact ? 'Expand' : 'Collapse'"
        class="absolute top-1/2 -translate-y-1/2 -right-3 z-[60] w-7 h-7 grid place-content-center rounded-full bg-white border border-emerald-200 text-emerald-700 shadow-[0_2px_10px_rgba(2,44,34,.22)] hover:bg-emerald-50 hover:border-emerald-300 hover:text-emerald-800 active:scale-95 transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500 focus-visible:ring-offset-2">
        <font-awesome :icon="sidebarCompact ? faChevronRight : faChevronLeft" class="w-3 h-3" />
      </button>
    </aside>

    <!-- ============ MAIN ============ -->
    <div class="flex-1 flex flex-col min-w-0 min-h-0 lg:h-screen lg:overflow-hidden">
      <header class="shrink-0 sticky top-0 z-30 glass border-b border-emerald-100">
        <div class="h-[2px] w-full bg-gradient-to-r from-emerald-500 via-lime-400 to-emerald-600 opacity-60"></div>
        <div class="flex h-[64px] items-center gap-3 px-4 sm:px-6">
          <!-- Mobile: Icon as Hamburger Menu-->
          <button type="button" @click="toggleMenu" :aria-expanded="openMenu ? 'true' : 'false'"
            aria-controls="mobile-nav" aria-label="Toggle navigation"
            class="lg:hidden w-9 h-9 rounded-xl overflow-hidden shadow-md ring-1 ring-emerald-100 hover:ring-emerald-200 active:scale-95 transition shrink-0">
            <img src="/lulu-192.png" alt="LuLu Attendance" class="w-full h-full object-cover">
          </button>
          <div class="min-w-0">
            <h1 class="display text-base sm:text-lg font-extrabold tracking-tight text-green-900 truncate">{{ pageTitle
              }}
            </h1>
            <p class="hidden sm:block mono text-[10px] tracking-[0.14em] text-emerald-700/50">LULU ATTENDANCE SYSTEM</p>
          </div>

          <div class="ml-auto flex items-center gap-2 sm:gap-3 min-w-0">

            <div ref="userMenuRef" class="relative flex items-center min-w-0">
              <button type="button" @click.stop="toggleUserMenu" :aria-expanded="showUserMenu ? 'true' : 'false'"
                aria-haspopup="menu" aria-label="User menu"
                class="flex items-center gap-2 rounded-xl px-1.5 py-1 hover:bg-emerald-50 active:bg-emerald-100 transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500 min-w-0">
                <div
                  class="w-8 h-8 rounded-full bg-gradient-to-br from-emerald-400 to-green-600 grid place-content-center text-white text-[11px] font-bold shadow-sm shrink-0">
                  {{ initials }}</div>
                <div class="hidden lg:block text-left leading-tight min-w-0">
                  <p class="text-xs font-semibold text-green-900 truncate max-w-[9rem]">{{ userName }}</p>
                </div>
                <font-awesome :icon="faChevronDown" class="hidden sm:block w-3 h-3 text-emerald-700/40 transition"
                  :class="showUserMenu ? 'rotate-180' : ''" />
              </button>
              <!-- Dropdown logout -->
              <div v-if="showUserMenu"
                class="absolute right-0 top-full mt-2 w-56 rounded-xl bg-white border border-emerald-100 shadow-[0_12px_32px_-12px_rgba(2,44,34,.28)] overflow-hidden z-50"
                role="menu" aria-label="User menu">
                <div class="px-4 py-3 bg-emerald-50/60 border-b border-emerald-100">
                  <p class="text-sm font-semibold text-green-900 truncate">{{ userName }}</p>
                </div>
                <button @click.stop="modalLogout" type="button" role="menuitem"
                  class="flex w-full items-center gap-3 px-4 py-3 text-sm font-medium text-emerald-900 hover:bg-red-50 hover:text-red-600 transition text-left">
                  <font-awesome :icon="faArrowRightFromBracket" class="w-4 h-4 shrink-0" />
                  <span>Log out</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </header>

      <!-- Mobile nav sheet -->
      <div v-if="openMenu" id="mobile-nav" class="lg:hidden fixed inset-0 z-[80]" role="dialog" aria-modal="true">
        <div class="absolute inset-0 bg-black/40 backdrop-blur-[2px]" @click="closeMenu"></div>
        <nav class="absolute start-0 top-0 bottom-0 w-[272px] max-w-[82vw] glass-dark flex flex-col shadow-2xl">
          <div class="flex items-center gap-3 h-[68px] px-4 border-b border-white/10 shrink-0">
            <div class="w-9 h-9 rounded-xl overflow-hidden ring-1 ring-white/20 shrink-0">
              <img src="/lulu-192.png" alt="LuLu Attendance" class="w-full h-full object-cover">
            </div>
            <div class="min-w-0 leading-tight">
              <p class="display text-sm font-extrabold text-white">LuLu</p>
              <p class="mono text-[10px] tracking-[0.14em] text-emerald-200/60">ATTENDANCE</p>
            </div>
            <button type="button" @click="closeMenu" aria-label="Close navigation"
              class="ms-auto w-9 h-9 grid place-content-center rounded-lg text-emerald-100 hover:text-white hover:bg-white/10 transition shrink-0">
              <font-awesome :icon="faXmark" class="w-4 h-4" />
            </button>
          </div>
          <div class="flex-1 overflow-y-auto overflow-x-hidden px-3 py-4 space-y-1">
            <p class="mono text-[10px] font-bold tracking-[0.18em] text-emerald-200/40 px-3 pb-2">MENU</p>
            <a v-for="n in NAV" :key="n.path" :href="n.path" @click.prevent="go(n.path)"
              class="flex items-center gap-3 px-3 py-3 rounded-xl font-semibold text-sm transition cursor-pointer"
              :class="isActive(n.path) ? 'bg-emerald-600 text-white shadow-sm' : 'text-white/90 hover:bg-white/10 border border-transparent'">
              <font-awesome :icon="n.icon" class="w-4 h-4 shrink-0" />
              <span class="truncate">{{ n.label }}</span>
            </a>
            <div class="my-3 h-px bg-white/10"></div>
            <a href="/settings" @click.prevent="go('/settings')"
              class="flex items-center gap-3 px-3 py-3 rounded-xl font-semibold text-sm transition cursor-pointer"
              :class="isActive('/settings') ? 'bg-white/15 text-white' : 'text-white/90 hover:bg-white/10 border border-transparent'">
              <font-awesome :icon="faSliders" class="w-4 h-4 shrink-0" />
              <span>Settings</span>
            </a>
          </div>
          <div class="border-t border-white/10 p-3 pb-[max(0.75rem,env(safe-area-inset-bottom))] shrink-0">
            <button @click="closeMenu(); modalLogout()" type="button"
              class="flex w-full items-center gap-3 px-3 py-3 rounded-xl font-semibold text-sm text-red-300 hover:bg-red-500/10 transition cursor-pointer">
              <font-awesome :icon="faArrowRightFromBracket" class="w-4 h-4 shrink-0" />
              <span>Log out</span>
            </button>
          </div>
        </nav>
      </div>

      <main class="flex-1 min-h-0 p-3 sm:p-4 lg:p-4 lg:overflow-y-auto lg:overflow-x-hidden flex flex-col">
        <slot />
      </main>
    </div>
  </div>

  <SimpleModal :options="sm_data.get()" v-on:cancel="actCancel" v-on:ok="actOk" v-show="sm_data.isShow()">
    {{ sm_data.message() }}
  </SimpleModal>
</template>
