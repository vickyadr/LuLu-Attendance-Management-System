<script setup>
import { useAuthStore } from '~/store/auth';
import { useNotifyStore } from '~/store/notify';
import { faEye, faEyeSlash, faArrowRight, faCircleNotch, faCircleExclamation, faXmark } from '@fortawesome/free-solid-svg-icons';

const
  validation = reactive({ user_id: '', password: '', }),
  user = reactive({ user_id: '', password: '' }),
  auth = useAuthStore(),
  notify = useNotifyStore();

const { t, locale } = useI18n();
const loading = ref(false);
const show_password = ref(false);
const remember = ref(true);

// Only the static demo build (mock API) shows the accounts: the real app must
// never advertise credentials on its login page.
const demoMode = computed(() => {
  try {
    const flag = useRuntimeConfig().public.demoMode
    return flag === true || flag === 'true'
  } catch {
    return false
  }
})
const DEMO_ACCOUNTS = [
  { name: 'demo', password: 'demo12345', role: 'login.demoRoleAdmin' },
  { name: 'auditor', password: 'audit12345', role: 'login.demoRoleViewer' },
  { name: 'staf', password: 'staf12345', role: 'login.demoRoleStaff' },
]
function fillDemo(account) {
  user.user_id = account.name
  user.password = account.password
  validation.user_id = undefined
  validation.password = undefined
}

useHead({ title: "LuLu's System - Login Page" });
definePageMeta({ middleware: ["get-auth"], layout: 'none' });

// NOTE: login logic is kept EXACTLY as the GitHub version that is verified working
const login = async () => {
  loading.value = true;
  try { if (remember.value && user.user_id.trim()) localStorage.setItem('lulu:last_user_id', user.user_id.trim()); else if (!remember.value) localStorage.removeItem('lulu:last_user_id'); } catch (e) { }
  await auth.attemptLogin({ nickname: user.user_id, password: user.password, cookies: true }).then((response) => {
    if (response.code == 200) {
      notify.success(t('login.successTitle'), t('login.redirecting'))
      setTimeout(() => { navigateTo("/", { replace: true }); }, 1200)
    } else {
      validation.user_id = response.data.nickname;
      validation.password = response.data.password;
      notify.error(t('login.failedTitle'), response.message)
    }
  }).finally(() => { loading.value = false })
}
// The field-level hints clear once the user starts typing again.
watch(() => [user.user_id, user.password], () => {
  validation.user_id = undefined; validation.password = undefined;
})

const capsOn = ref(false);
const canSubmit = computed(() => user.user_id.trim().length >= 2 && user.password.length >= 3 && !loading.value);
const hasUser = computed(() => user.user_id.trim().length > 0);
const hasPass = computed(() => user.password.length > 0);
const submitTitle = computed(() => {
  if (loading.value) return t('login.signingIn');
  if (!hasUser.value) return t('login.userIdPlaceholder');
  if (user.password.length < 3) return t('login.passwordMinChars');
  return `${t('login.signIn')} (↵)`;
});
function clearUser() {
  user.user_id = ''; validation.user_id = undefined; capsOn.value = false;
  nextTick(() => document.getElementById('user_name')?.focus());
}
function onCapsCheck(e) { try { capsOn.value = e.getModifierState && e.getModifierState('CapsLock'); } catch (_) { } }
function onPassBlur() { capsOn.value = false; }

const greeting = computed(() => {
  const h = new Date().getHours();
  if (h < 11) return t('greeting.morning');
  if (h < 15) return t('greeting.afternoon');
  if (h < 18) return t('greeting.evening');
  return t('greeting.night');
});
const nowStr = ref('');
let clockIv = null;
function tickClock() {
  try {
    const d = new Date();
    nowStr.value = new Intl.DateTimeFormat(locale.value, { weekday: 'short', day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' }).format(d) + ' WIB';
  } catch { nowStr.value = ''; }
}
function onCardMove(e) {
  const r = e.currentTarget.getBoundingClientRect();
  const x = ((e.clientX - r.left) / r.width) * 100;
  const y = ((e.clientY - r.top) / r.height) * 100;
  e.currentTarget.style.setProperty('--mx', x + '%');
  e.currentTarget.style.setProperty('--my', y + '%');
}

onMounted(() => {
  tickClock(); clockIv = setInterval(tickClock, 30000);
  try {
    const last = localStorage.getItem('lulu:last_user_id');
    if (last && !user.user_id) user.user_id = last;
    const rm = localStorage.getItem('lulu:remember');
    if (rm === '0') remember.value = false;
  } catch (e) { }
  nextTick(() => {
    const el = !user.user_id.trim() ? document.getElementById('user_name') : document.getElementById('user_password');
    el?.focus();
    if (typeof window !== 'undefined' && window.matchMedia('(pointer: coarse)').matches) (document.activeElement)?.blur();
    const card = document.getElementById('login-card');
    if (card) { card.style.setProperty('--mx', '60%'); card.style.setProperty('--my', '28%'); }
  });
});
onBeforeUnmount(() => { if (clockIv) clearInterval(clockIv); });
watch(remember, (v) => { try { localStorage.setItem('lulu:remember', v ? '1' : '0'); } catch (e) { } });
</script>

<template>
  <div
    class="min-h-[100dvh] grid place-items-center bg-canvas relative overflow-x-hidden px-4 py-8 selection:bg-surface-sunken selection:text-text">

    <!-- One ambient layer: a single wash and a faint grid. No orb soup. -->
    <div class="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
      <div class="absolute inset-0 bg-gradient-to-br from-surface-raised via-surface to-surface"></div>
      <div class="absolute inset-0 tech-grid-soft opacity-[0.35]"></div>
      <div class="absolute -top-40 -right-32 w-[34rem] h-[34rem] rounded-full bg-emerald-200/25 blur-[120px]"></div>
      <div class="absolute -bottom-48 -left-32 w-[30rem] h-[30rem] rounded-full bg-lime-200/20 blur-[120px]"></div>
    </div>

    <div class="relative w-full max-w-[420px]">

      <!-- Brand lockup -->
      <div class="mb-5 flex items-center justify-center gap-2.5">
        <span
          class="w-9 h-9 rounded-xl overflow-hidden ring-1 ring-border-strong bg-surface grid place-content-center shrink-0">
          <img src="/android-chrome-512x512.png" alt="LuLu" class="w-full h-full object-cover" />
        </span>
        <div class="leading-none">
          <p class="display text-[15px] text-text">LuLu</p>
          <p class="mono text-[9.5px] tracking-[0.18em] text-text-muted mt-1">{{ $t('nav.subtitle') }}</p>
        </div>
      </div>

      <div id="login-card"
        class="relative w-full rounded-[20px] bg-surface border border-border shadow-tint-lg overflow-hidden">
        <div class="h-[3px] w-full bg-gradient-to-r from-emerald-500 via-lime-400 to-emerald-600"></div>

        <div class="px-5 sm:px-7 pt-6 pb-6">
          <h1 class="display text-[24px] text-text leading-none">{{ $t('login.welcomeBack') }}</h1>
          <p class="text-[13px] leading-relaxed text-text-muted mt-2">{{ $t('login.hint') }}</p>

          <form @submit.prevent="login" class="mt-6" novalidate>
            <!-- User id -->
            <div>
              <label for="user_name" class="label">{{ $t('login.userId') }}</label>
              <div class="relative">
                <input v-model.trim="user.user_id" id="user_name" type="text" required autocomplete="username"
                  autocorrect="off" spellcheck="false" inputmode="text" enterkeyhint="next"
                  :placeholder="$t('login.userIdPlaceholder')" :disabled="loading" :aria-invalid="!!validation.user_id"
                  :aria-describedby="validation.user_id ? 'err-user' : undefined"
                  @keydown.enter.prevent="() => { if (canSubmit) login(); else document.getElementById('user_password')?.focus(); }"
                  :class="['field field-lg has-icon-right',
                    validation.user_id ? 'border-red-300 focus:border-red-400' : '',
                    hasUser && !validation.user_id ? 'border-emerald-300' : '']">
                <button v-if="hasUser && !loading" type="button" @click="clearUser" tabindex="-1"
                  class="absolute right-1 top-1/2 -translate-y-1/2 grid w-10 h-10 place-content-center rounded-lg text-text-muted hover:text-primary-deeper hover:bg-surface-raised active:scale-95 transition"
                  :aria-label="$t('login.clearUserId')">
                  <font-awesome :icon="faXmark" class="w-4 h-4" />
                </button>
              </div>
              <div class="min-h-[20px] pt-1.5">
                <p v-if="validation.user_id" id="err-user" role="alert"
                  class="flex items-center gap-1.5 text-[12.5px] font-medium text-danger">
                  <font-awesome :icon="faCircleExclamation" class="w-3.5 h-3.5 shrink-0" />
                  {{ validation.user_id }}
                </p>
              </div>
            </div>

            <!-- Password -->
            <div>
              <label for="user_password" class="label">{{ $t('login.password') }}</label>
              <div class="relative">
                <input v-model="user.password" id="user_password" :type="show_password ? 'text' : 'password'" required
                  autocomplete="current-password" enterkeyhint="done" placeholder="••••••••" :disabled="loading"
                  :aria-invalid="!!validation.password"
                  :aria-describedby="validation.password ? 'err-pass' : (capsOn ? 'capswarn' : undefined)"
                  @keyup="onCapsCheck" @keydown="onCapsCheck" @blur="onPassBlur" @keydown.enter.prevent="login" :class="['field field-lg has-icon-right',
                    validation.password ? 'border-red-300 focus:border-red-400' : '',
                    hasPass && !validation.password ? 'border-emerald-300' : '']">
                <button type="button" @click="show_password = !show_password" :disabled="loading"
                  class="absolute right-1 top-1/2 -translate-y-1/2 grid w-10 h-10 place-content-center rounded-lg text-text-muted hover:text-primary-deeper hover:bg-surface-raised active:scale-95 transition disabled:opacity-40"
                  :aria-label="show_password ? $t('login.hidePassword') : $t('login.showPassword')"
                  :aria-pressed="show_password ? 'true' : 'false'">
                  <font-awesome :icon="show_password ? faEyeSlash : faEye" class="w-[18px] h-[18px]" />
                </button>
              </div>
              <div class="min-h-[20px] pt-1.5">
                <p v-if="validation.password" id="err-pass" role="alert"
                  class="flex items-center gap-1.5 text-[12.5px] font-medium text-danger">
                  <font-awesome :icon="faCircleExclamation" class="w-3.5 h-3.5 shrink-0" />
                  {{ validation.password }}
                </p>
                <p v-else-if="capsOn" id="capswarn" role="status"
                  class="flex items-center gap-1.5 text-[12.5px] font-medium text-warn">
                  <font-awesome :icon="faCircleExclamation" class="w-3.5 h-3.5 shrink-0" />
                  {{ $t('login.capsLockOn') }}
                </p>
              </div>
            </div>

            <div class="mt-1 flex items-center justify-between gap-3">
              <label
                class="inline-flex items-center gap-2.5 cursor-pointer select-none min-h-[44px] -ml-0.5 pr-1 rounded-lg">
                <input v-model="remember" type="checkbox" class="peer sr-only"
                  :aria-label="$t('login.rememberUserId')" />
                <span class="grid h-5 w-5 place-content-center rounded-md border-2 transition"
                  :class="remember ? 'bg-primary-deep border-primary-deep text-white' : 'bg-surface border-border-strong'">
                  <svg v-if="remember" xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="none"
                    viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                  </svg>
                </span>
                <span class="text-[13px] font-semibold" :class="remember ? 'text-text' : 'text-text-muted'">{{
                  $t('login.rememberMe') }}</span>
              </label>

              <span class="mono text-[10.5px] text-text-muted" aria-live="polite">
                <span v-if="canSubmit" class="inline-flex items-center gap-1.5">{{ $t('login.pressEnterA') }}
                  <kbd
                    class="rounded-md border border-border-strong bg-surface px-1.5 py-0.5 text-[10.5px] font-bold shadow-tint-sm">↵</kbd>
                  {{ $t('login.pressEnterB') }}</span>
              </span>
            </div>

            <button type="submit" :disabled="loading || !canSubmit" :title="submitTitle"
              :aria-busy="loading ? 'true' : 'false'"
              class="btn btn-primary w-full !h-12 !min-h-12 mt-4 text-[14px] tracking-wide">
              <template v-if="loading">
                <font-awesome :icon="faCircleNotch" class="w-4 h-4 animate-spin" />
                {{ $t('login.signingIn') }}
              </template>
              <template v-else>
                {{ $t('login.signIn') }}
                <font-awesome :icon="faArrowRight" class="w-4 h-4" />
              </template>
            </button>
          </form>

          <!-- Demo accounts, only on the static demo build -->
          <div v-if="demoMode" class="mt-5 rounded-xl border border-border bg-surface-raised/50 p-3">
            <p class="mono text-[9.5px] font-bold tracking-[0.14em] uppercase text-text-muted">{{
              $t('login.demoAccounts') }}</p>
            <p class="text-[11.5px] text-text-muted mt-1">{{ $t('login.demoFillHint') }}</p>
            <div class="mt-2 grid gap-1.5 sm:grid-cols-3">
              <button v-for="account in DEMO_ACCOUNTS" :key="account.name" type="button"
                class="rounded-lg border border-border bg-surface px-2.5 py-2 text-left transition hover:border-emerald-300 hover:shadow-tint-sm active:scale-[0.99]"
                @click="fillDemo(account)">
                <span class="block mono text-[11px] font-bold text-text">{{ account.name }}</span>
                <span class="block mono text-[10px] text-text-muted">{{ account.password }}</span>
                <span class="block text-[10px] text-text-muted mt-0.5">{{ $t(account.role) }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer: three separate items so the copyright never breaks apart -->
      <div
        class="mt-4 flex flex-wrap items-center justify-center gap-x-2 gap-y-1 mono text-[9.5px] tracking-[0.12em] text-text-muted">
        <span>{{ greeting }}</span>
        <span class="opacity-40" aria-hidden="true">|</span>
        <span>{{ nowStr || '-' }}</span>
        <span class="opacity-40" aria-hidden="true">|</span>
        <span class="whitespace-nowrap">&copy; 2026 LuLu</span>
      </div>
    </div>
  </div>

  <ToastStack />
</template>
