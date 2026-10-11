<script setup>
import { faExclamationTriangle, faExclamationCircle, faLock, faSearch, faClock, faServer, faWifi, faCreditCard, faTachometerAlt } from '@fortawesome/free-solid-svg-icons'

const props = defineProps({
  code: { type: Number, required: true },
  title: { type: String, default: '' },
  message: { type: String, default: '' },
  actionLabel: { type: String, default: '' },
  actionTo: { type: String, default: '/' },
  showRetry: { type: Boolean, default: false },
})

const CODE_META = {
  400: { icon: faExclamationTriangle, label: $t('error.badgeBadRequest'), title: $t('error.tBadRequest'), message: $t('error.mBadRequest') },
  401: { icon: faLock, label: $t('error.badgeUnauthorized'), title: $t('error.tUnauthorized'), message: $t('error.mUnauthorized') },
  402: { icon: faCreditCard, label: $t('error.badgePayment'), title: $t('error.tPayment'), message: $t('error.mPayment') },
  403: { icon: faLock, label: $t('error.badgeForbidden'), title: $t('error.tForbidden'), message: $t('error.mForbidden') },
  404: { icon: faSearch, label: $t('error.badgeNotFound'), title: $t('error.tNotFound'), message: $t('error.mNotFound') },
  405: { icon: faExclamationCircle, label: $t('error.badgeMethod'), title: $t('error.tMethod'), message: $t('error.mMethod') },
  408: { icon: faClock, label: $t('error.badgeTimeout'), title: $t('error.tTimeout'), message: $t('error.mTimeout') },
  429: { icon: faExclamationCircle, label: $t('error.badgeTooMany'), title: $t('error.tTooMany'), message: $t('error.mTooMany') },
  500: { icon: faServer, label: $t('error.badgeServer'), title: $t('error.tServer'), message: $t('error.mServer') },
  502: { icon: faWifi, label: $t('error.badgeGateway'), title: $t('error.tGateway'), message: $t('error.mGateway') },
  503: { icon: faTachometerAlt, label: $t('error.badgeUnavailable'), title: $t('error.tUnavailable'), message: $t('error.mUnavailable') },
  504: { icon: faWifi, label: $t('error.badgeGatewayTimeout'), title: $t('error.tGatewayTimeout'), message: $t('error.mGatewayTimeout') },
}
const RETRY_CODES = [408, 429, 500, 502, 503, 504]
const fallback = { icon: faExclamationCircle, label: $t('error.badgeUnknown'), title: $t('error.tUnknown'), message: 'An unexpected error occurred.' }

const meta = computed(() => CODE_META[props.code] || fallback)
const resolvedTitle = computed(() => props.title || meta.value.title)
const resolvedMessage = computed(() => props.message || meta.value.message)
const resolvedAction = computed(() => props.actionLabel || (meta.value.action || $t('error.backToDashboard')))
const resolvedActionTo = computed(() => {
  if (props.actionTo !== '/') return props.actionTo
  return [401, 403].includes(props.code) ? '/login' : '/'
})
const resolvedShowRetry = computed(() => props.showRetry || RETRY_CODES.includes(props.code))

function goBack() { navigateTo(resolvedActionTo.value, { replace: true }) }
function retry() { window.location.reload() }
</script>

<template>
  <div class="min-h-[60vh] grid place-content-center text-center p-8 tech-grid-soft relative">
    <div
      class="hud-frame glass rounded-[2rem] p-10 border border-emerald-100 max-w-md mx-auto relative overflow-hidden">
      <span class="hud-corner hud-corner-tl"></span>
      <span class="hud-corner hud-corner-br"></span>

      <div class="mono text-[10px] tracking-widest text-emerald-700/40 mb-2">
        ERR {{ code }} &bull; {{ meta.label }}
      </div>

      <div
        class="mx-auto w-14 h-14 rounded-xl bg-emerald-50 border border-emerald-200 flex items-center justify-center text-emerald-600 mb-4">
        <font-awesome :icon="meta.icon" class="w-7 h-7" />
      </div>

      <h1 class="text-4xl font-extrabold text-green-900">{{ code }}</h1>
      <p class="text-emerald-700/70 mt-2 font-medium">{{ resolvedTitle }}</p>
      <p class="text-sm text-emerald-700/50 mt-1">{{ resolvedMessage }}</p>

      <div class="mt-6 flex flex-col sm:flex-row items-center justify-center gap-3">
        <button v-if="resolvedShowRetry" @click="retry" class="btn-emerald px-6 py-2.5 rounded-xl text-sm font-bold">
          {{ $t('common.retry') }}
        </button>
        <button @click="goBack"
          class="px-6 py-2.5 rounded-xl text-sm font-bold border-2 border-emerald-200 text-emerald-700 hover:bg-emerald-50 transition">
          {{ resolvedAction }}
        </button>
      </div>
    </div>
  </div>
</template>
