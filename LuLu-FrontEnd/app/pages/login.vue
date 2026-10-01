<script setup>
import { useAuthStore } from '~/store/auth';
import { SMData } from '~/components/SimpleModal.vue';

const
    validation = reactive({ user_id: '', password: '', }),
    user = reactive({ user_id: '', password: '' }),
    auth = useAuthStore(),
    sm_data = new SMData();

const loading = ref(false);
const show_password = ref(false);
const remember = ref(true);

useHead({ title: "LuLu's System - Login Page" });
definePageMeta({ middleware: ["get-auth"], layout: 'none' });

const login = async () => {
    loading.value = true;
    try { if (remember.value && user.user_id.trim()) localStorage.setItem('lulu:last_user_id', user.user_id.trim()); else if (!remember.value) localStorage.removeItem('lulu:last_user_id'); } catch (e) { }
    await auth.attemptLogin({ nickname: user.user_id, password: user.password, cookies: true }).then((response) => {
        if (response.code == 200) {
            sm_data.setText("Login", "Login successful, please wait…'re tryng to redirecting the page")
            sm_data.showOK()
            setTimeout(() => { navigateTo("/", { replace: true }); sm_data.clear(); }, 3000)
        } else {
            validation.user_id = response.data.nickname;
            validation.password = response.data.password;
            sm_data.setText("Login", response.message)
            sm_data.showOK()
        }
    }).finally(() => { loading.value = false })
}
function actOk() {
    sm_data.clear()
    setTimeout(() => { validation.user_id = undefined; validation.password = undefined; }, 3000);
}

const capsOn = ref(false);
const canSubmit = computed(() => user.user_id.trim().length >= 2 && user.password.length >= 3 && !loading.value);
const hasUser = computed(() => user.user_id.trim().length > 0);
const hasPass = computed(() => user.password.length > 0);
const submitTitle = computed(() => {
    if (loading.value) return 'Signing in…';
    if (!hasUser.value) return 'Enter your User ID';
    if (user.password.length < 3) return 'Password minimal 3 karakter';
    return 'Sign in (Enter)';
});
function clearUser() {
    user.user_id = ''; validation.user_id = undefined; capsOn.value = false;
    nextTick(() => document.getElementById('user_name')?.focus());
}
function onCapsCheck(e) { try { capsOn.value = e.getModifierState && e.getModifierState('CapsLock'); } catch (_) { } }
function onPassBlur() { capsOn.value = false; }

const greeting = computed(() => {
    const h = new Date().getHours();
    if (h < 11) return 'Good morning';
    if (h < 15) return 'Good afternoon';
    if (h < 18) return 'Good evening';
    return 'Good night';
});
const nowStr = ref('');
let clockIv = null;
function tickClock() {
    try {
        const d = new Date();
        nowStr.value = new Intl.DateTimeFormat('en-GB', { weekday: 'short', day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' }).format(d) + ' WIB';
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
        class="min-h-[100dvh] flex flex-col items-center justify-center bg-[#f6fdf7] relative overflow-x-hidden selection:bg-emerald-200 selection:text-emerald-900 px-4 py-4 sm:px-6 sm:py-6">

        <div class="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
            <div class="absolute inset-0 bg-gradient-to-br from-emerald-50 via-white to-teal-50"></div>
            <div class="absolute inset-0 tech-grid-soft opacity-[0.08] sm:opacity-[0.14]"></div>
            <div
                class="absolute -top-10 -right-10 sm:-top-32 sm:-right-32 w-[10rem] h-[10rem] sm:w-[26rem] sm:h-[26rem] rounded-full bg-gradient-to-br from-emerald-200/15 sm:from-emerald-200/30 via-lime-200/8 sm:via-lime-200/15 to-transparent blur-3xl">
            </div>
            <div
                class="absolute -bottom-10 -left-10 sm:-bottom-32 sm:-left-32 w-[10rem] h-[10rem] sm:w-[20rem] sm:h-[20rem] rounded-full bg-gradient-to-tr from-emerald-300/8 sm:from-emerald-300/15 via-teal-200/5 sm:via-teal-200/10 to-transparent blur-3xl">
            </div>
            <div class="absolute inset-0 opacity-20 sm:opacity-60">
                <div class="absolute left-0 right-0 h-px bg-gradient-to-r from-transparent via-emerald-300/25 sm:via-emerald-300/50 to-transparent"
                    style="top: 22%; animation: scan 4.6s linear infinite;"></div>
                <div class="absolute left-0 right-0 h-px bg-gradient-to-r from-transparent via-lime-200/15 sm:via-lime-200/35 to-transparent"
                    style="top: 68%; animation: scan 5.8s linear infinite 1s;"></div>
            </div>
            <div
                class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[220px] h-[220px] sm:w-[420px] sm:h-[420px] rounded-full bg-gradient-to-br from-emerald-200/8 sm:from-emerald-200/15 via-teal-100/5 sm:via-teal-100/8 to-transparent blur-3xl">
            </div>
        </div>
        <div class="relative w-full max-w-[420px] flex flex-col items-center">

            <div
                class="mb-1 sm:mb-3 inline-flex items-center gap-1.5 sm:gap-2 px-2.5 sm:px-3 py-1 sm:py-1.5 rounded-full bg-white border border-emerald-100 shadow-sm">
                <span
                    class="w-6 h-6 sm:w-7 sm:h-7 rounded-lg overflow-hidden ring-1 ring-emerald-200 bg-white grid place-content-center shrink-0">
                    <img src="/android-chrome-512x512.png" alt="LuLu" class="w-full h-full object-cover" />
                </span>
                <span class="display text-[12px] font-extrabold tracking-tight text-green-900">LuLu</span>
                <span class="mono text-[10px] tracking-[0.16em] text-emerald-700/50">ATTENDANCE</span>
                <span
                    class="hidden sm:inline-flex items-center gap-1 ml-1 px-2 py-0.5 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 mono text-[9px] font-bold tracking-widest"><span
                        class="w-1 h-1 rounded-full bg-emerald-500 animate-pulse"></span> SECURE</span>
            </div>

            <div id="login-card" @mousemove="onCardMove"
                class="relative w-full max-w-[420px] rounded-[16px] sm:rounded-[24px] bg-white/95 backdrop-blur border border-emerald-100 shadow-[0_16px_40px_-16px_rgba(2,44,34,.20),0_6px_16px_-10px_rgba(2,44,34,.10)] sm:shadow-[0_20px_60px_-20px_rgba(2,44,34,.22),0_8px_20px_-12px_rgba(2,44,34,.12)] overflow-hidden"
                style="--mx:60%; --my:28%;">
                <!-- spotlight -->
                <div class="pointer-events-none absolute -inset-px rounded-[16px] sm:rounded-[24px] opacity-100 transition-opacity"
                    style="background: radial-gradient(420px circle at var(--mx) var(--my), rgba(16,185,129,.14), transparent 62%);">
                </div>
                <div
                    class="pointer-events-none absolute inset-0 rounded-[16px] sm:rounded-[24px] ring-1 ring-emerald-100/70">
                </div>
                <div class="h-1 w-full bg-gradient-to-r from-emerald-500 via-lime-400 to-emerald-600 opacity-90"></div>

                <div class="px-3 sm:px-7 pt-2.5 sm:pt-6 pb-2 sm:pb-5">
                    <div class="flex items-center justify-between gap-3">
                        <div>
                            <h1
                                class="display text-[16px] sm:text-[22px] font-extrabold tracking-tight text-green-900 leading-none">
                                Welcome back</h1>
                            <p class="text-[11.5px] sm:text-[13px] leading-relaxed text-emerald-800/55 mt-0.5 sm:mt-1">
                                Sign in with your registered User ID.</p>
                        </div>
                        <span
                            class="hidden sm:grid w-9 h-9 rounded-xl bg-emerald-50 border border-emerald-100 place-content-center text-emerald-600">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.7">
                                <path
                                    d="M12 3.5a6.5 6.5 0 0 0-6.5 6.5v2.2a6.5 6.5 0 0 0 6.5 6.5 6.5 6.5 0 0 0 6.5-6.5V10A6.5 6.5 0 0 0 12 3.5Z" />
                                <path
                                    d="M8.2 9.2c.6-1.1 1.9-1.9 3.8-1.9 1.9 0 3.2.8 3.8 1.9M9.1 12c.4-.7 1.3-1.2 2.9-1.2s2.5.5 2.9 1.2M10 14.6c.3-.4.9-.7 2-.7s1.7.3 2 .7"
                                    stroke-linecap="round" />
                                <circle cx="12" cy="14.8" r="1.15" fill="currentColor" />
                            </svg>
                        </span>
                    </div>

                    <form @submit.prevent="login" class="mt-2 sm:mt-6 space-y-0" novalidate>
                        <div>
                            <label for="user_name"
                                class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">User
                                ID</label>
                            <div class="relative mt-1.5 sm:mt-2 group">
                                <input v-model.trim="user.user_id" id="user_name" type="text" required
                                    autocomplete="username" autocorrect="off" spellcheck="false" inputmode="text"
                                    enterkeyhint="next" placeholder="Enter your User ID" :disabled="loading"
                                    :aria-invalid="!!validation.user_id"
                                    :aria-describedby="validation.user_id ? 'err-user' : undefined"
                                    @keydown.enter.prevent="() => { if (canSubmit) login(); else document.getElementById('user_password')?.focus(); }"
                                    :class="['w-full rounded-xl border bg-white px-4 pr-[52px] text-[16px] sm:text-[15px] font-medium text-green-900 placeholder:text-emerald-800/35 focus:outline-none focus:bg-white transition disabled:bg-gray-50 h-[36px] sm:h-12', validation.user_id ? 'border-red-300 focus:border-red-400 focus:ring-4 focus:ring-red-500/10 shake' : hasUser ? 'border-emerald-300 focus:border-emerald-500 focus:ring-4 focus:ring-emerald-500/10' : 'border-emerald-200 focus:border-emerald-500 focus:ring-4 focus:ring-emerald-500/10']">
                                <button v-if="hasUser && !loading" type="button" @click="clearUser" tabindex="-1"
                                    class="absolute right-1 top-1/2 -translate-y-1/2 grid h-8 w-8 sm:h-11 sm:w-11 place-content-center rounded-xl text-emerald-600/60 hover:text-emerald-700 hover:bg-emerald-50 active:scale-95 transition"
                                    aria-label="Clear User ID">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none"
                                        viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                                    </svg>
                                </button>
                            </div>
                            <div class="min-h-[8px] sm:min-h-[18px] mt-0.5 sm:mt-1">
                                <p v-if="validation.user_id" id="err-user" role="alert"
                                    class="flex items-center gap-1.5 text-[12.5px] sm:text-[13px] font-medium text-red-600">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 shrink-0" fill="none"
                                        viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                    {{ validation.user_id }}
                                </p>
                            </div>
                        </div>

                        <div class="mt-0.5 sm:mt-1">
                            <label for="user_password"
                                class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">Password</label>
                            <div class="relative mt-1.5 sm:mt-2">
                                <input v-model="user.password" id="user_password"
                                    :type="show_password ? 'text' : 'password'" required autocomplete="current-password"
                                    enterkeyhint="done" placeholder="••••••••" :disabled="loading"
                                    :aria-invalid="!!validation.password"
                                    :aria-describedby="validation.password ? 'err-pass' : (capsOn ? 'capswarn' : undefined)"
                                    @keyup="onCapsCheck" @keydown="onCapsCheck" @blur="onPassBlur"
                                    @keydown.enter.prevent="login"
                                    :class="['w-full rounded-xl border bg-white px-4 pr-[52px] text-[16px] sm:text-[15px] font-medium text-green-900 placeholder:text-emerald-800/35 focus:outline-none focus:bg-white transition disabled:bg-gray-50 h-[36px] sm:h-12', validation.password ? 'border-red-300 focus:border-red-400 focus:ring-4 focus:ring-red-500/10 shake' : hasPass ? 'border-emerald-300 focus:border-emerald-500 focus:ring-4 focus:ring-emerald-500/10' : 'border-emerald-200 focus:border-emerald-500 focus:ring-4 focus:ring-emerald-500/10']">
                                <button type="button" @click="show_password = !show_password" :disabled="loading"
                                    class="absolute right-1 top-1/2 -translate-y-1/2 grid h-8 w-8 sm:h-11 sm:w-11 place-content-center rounded-xl text-emerald-700 hover:bg-emerald-50 active:scale-95 transition disabled:opacity-40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500"
                                    :aria-label="show_password ? 'Hide password' : 'Show password'"
                                    :aria-pressed="show_password ? 'true' : 'false'">
                                    <svg v-if="!show_password" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5"
                                        fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.7">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.964-7.178z" />
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                    </svg>
                                    <svg v-else xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none"
                                        viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.7">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.522 10.522 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
                                    </svg>
                                </button>
                            </div>
                            <div class="min-h-[8px] sm:min-h-[18px] mt-0.5 sm:mt-1">
                                <p v-if="validation.password" id="err-pass" role="alert"
                                    class="flex items-center gap-1.5 text-[12.5px] sm:text-[13px] font-medium text-red-600">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 shrink-0" fill="none"
                                        viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                    {{ validation.password }}
                                </p>
                                <p v-else-if="capsOn" id="capswarn" role="status"
                                    class="flex items-center gap-1.5 text-[12.5px] sm:text-[13px] font-medium text-amber-600">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 shrink-0" fill="none"
                                        viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                            d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
                                    </svg>
                                    Caps Lock is on
                                </p>
                            </div>
                        </div>

                        <div class="mt-0.5 sm:mt-3 flex items-center justify-between gap-3">
                            <label
                                class="inline-flex items-center gap-2 sm:gap-2.5 cursor-pointer select-none min-h-[28px] sm:min-h-[44px] -ml-1 px-1 rounded-lg focus-within:ring-2 focus-within:ring-emerald-500/20">
                                <input v-model="remember" type="checkbox" class="peer sr-only"
                                    aria-label="Remember User ID" />
                                <span class="grid h-5 w-5 place-content-center rounded-md border-2 transition"
                                    :class="remember ? 'bg-emerald-600 border-emerald-600 text-white' : 'bg-white border-emerald-200'">
                                    <svg v-if="remember" xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="none"
                                        viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                                    </svg>
                                </span>
                                <span class="text-[13px] font-medium leading-none"
                                    :class="remember ? 'text-green-900' : 'text-emerald-800/70'">Remember me</span>
                            </label>
                            <span class="mono text-[11px]"
                                :class="canSubmit ? 'text-emerald-600 font-medium' : 'text-emerald-700/40'">
                                <span v-if="canSubmit" class="inline-flex items-center gap-1">Press <kbd
                                        class="rounded-md border border-emerald-200 bg-white px-1.5 py-0.5 text-[11px] font-bold shadow-sm">↵</kbd>
                                    to sign in</span>
                                <span v-else class="hidden sm:inline">&nbsp;</span>
                            </span>
                        </div>

                        <button type="submit" :disabled="loading || !canSubmit" :title="submitTitle"
                            :aria-busy="loading ? 'true' : 'false'"
                            class="mt-1.5 sm:mt-4 flex w-full items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-emerald-600 to-emerald-600 hover:from-emerald-600 hover:to-emerald-700 px-4 text-[15px] font-bold tracking-wide text-white shadow-[0_10px_24px_-12px_rgba(5,150,105,.45)] transition min-h-[38px] sm:min-h-[48px] active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500 focus-visible:ring-offset-2 relative overflow-hidden group">
                            <span
                                class="pointer-events-none absolute inset-0 opacity-0 group-hover:opacity-100 transition bg-gradient-to-r from-white/0 via-white/10 to-white/0"
                                style="background-size: 200% 100%; animation: shine 1.2s linear infinite;"></span>
                            <span v-if="loading" class="relative inline-flex items-center gap-2">
                                <span class="inline-flex gap-1" aria-hidden="true">
                                    <span class="w-1.5 h-1.5 rounded-full bg-white animate-bounce"
                                        style="animation-delay:0ms"></span>
                                    <span class="w-1.5 h-1.5 rounded-full bg-white animate-bounce"
                                        style="animation-delay:140ms"></span>
                                    <span class="w-1.5 h-1.5 rounded-full bg-white animate-bounce"
                                        style="animation-delay:280ms"></span>
                                </span>
                                Signing in…
                            </span>
                            <span v-else class="relative inline-flex items-center gap-2">Sign in <svg
                                    xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                                    stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                                </svg></span>
                        </button>
                    </form>
                </div>
                <div
                    class="px-4 sm:px-7 py-1 sm:py-3 bg-emerald-50/50 border-t border-emerald-100 flex items-center justify-center gap-2 mono text-[10px] tracking-widest text-emerald-700/40">
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> SECURE • ENCRYPTED • BIOMETRIC READY
                </div>
            </div>

            <p class="mono text-center text-[9px] sm:text-[10px] tracking-[0.14em] text-emerald-700/30 mt-1 sm:mt-4">{{
                greeting }} • {{ nowStr || '—' }} • © 2026 LuLu</p>
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" v-on:ok="actOk" v-show="sm_data.isShow()">{{ sm_data.message() }}
    </SimpleModal>
</template>

<style scoped>
@keyframes scan {
    0% {
        transform: translateY(-12px);
        opacity: 0
    }

    20% {
        opacity: 1
    }

    80% {
        opacity: 1
    }

    100% {
        transform: translateY(140px);
        opacity: 0
    }
}

@keyframes shine {
    0% {
        background-position: 200% 0
    }

    100% {
        background-position: -200% 0
    }
}

@keyframes shake {

    0%,
    100% {
        transform: translateX(0)
    }

    20% {
        transform: translateX(-4px)
    }

    40% {
        transform: translateX(4px)
    }

    60% {
        transform: translateX(-3px)
    }

    80% {
        transform: translateX(3px)
    }
}

.shake {
    animation: shake .38s ease;
}

@media (prefers-reduced-motion: reduce) {
    .shake {
        animation: none
    }

    .animate-bounce {
        animation: none !important
    }

    [style*="animation: scan"] {
        animation: none !important
    }
}
</style>
