<script setup>
import { faExclamationTriangle, faExclamationCircle, faLock, faSearch, faClock, faServer, faWifi, faCreditCard, faTachometerAlt } from '@fortawesome/free-solid-svg-icons'

const { t } = useI18n()

const props = defineProps({
  code: { type: Number, required: true },
  title: { type: String, default: '' },
  message: { type: String, default: '' },
  actionLabel: { type: String, default: '' },
  actionTo: { type: String, default: '/' },
  showRetry: { type: Boolean, default: false },
})

const CODE_META = {
  400: { icon: faExclamationTriangle, label: 'error.badgeBadRequest', title: 'error.tBadRequest', message: 'error.mBadRequest' },
  401: { icon: faLock, label: 'error.badgeUnauthorized', title: 'error.tUnauthorized', message: 'error.mUnauthorized' },
  402: { icon: faCreditCard, label: 'error.badgePayment', title: 'error.tPayment', message: 'error.mPayment' },
  403: { icon: faLock, label: 'error.badgeForbidden', title: 'error.tForbidden', message: 'error.mForbidden' },
  404: { icon: faSearch, label: 'error.badgeNotFound', title: 'error.tNotFound', message: 'error.mNotFound' },
  405: { icon: faExclamationCircle, label: 'error.badgeMethod', title: 'error.tMethod', message: 'error.mMethod' },
  408: { icon: faClock, label: 'error.badgeTimeout', title: 'error.tTimeout', message: 'error.mTimeout' },
  429: { icon: faExclamationCircle, label: 'error.badgeTooMany', title: 'error.tTooMany', message: 'error.mTooMany' },
  500: { icon: faServer, label: 'error.badgeServer', title: 'error.tServer', message: 'error.mServer' },
  502: { icon: faWifi, label: 'error.badgeGateway', title: 'error.tGateway', message: 'error.mGateway' },
  503: { icon: faTachometerAlt, label: 'error.badgeUnavailable', title: 'error.tUnavailable', message: 'error.mUnavailable' },
  504: { icon: faWifi, label: 'error.badgeGatewayTimeout', title: 'error.tGatewayTimeout', message: 'error.mGatewayTimeout' },
}
const RETRY_CODES = [408, 429, 500, 502, 503, 504]
const fallback = { icon: faExclamationCircle, label: 'error.badgeUnknown', title: 'error.tUnknown', message: 'An unexpected error occurred.' }

const meta = computed(() => CODE_META[props.code] || fallback)
const resolvedTitle = computed(() => props.title || $t(meta.value.title))
const resolvedMessage = computed(() => props.message || $t(meta.value.message))
const resolvedAction = computed(() => props.actionLabel || t('error.backToDashboard'))
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

      <div class="mono text-[10px] tracking-widest text-emerald-700 mb-2">
        ERR {{ code }} &bull; {{ $t(meta.label) }}
      </div>

      <div
        class="mx-auto w-14 h-14 rounded-xl bg-emerald-50 border border-emerald-200 flex items-center justify-center text-emerald-600 mb-4">
        <font-awesome :icon="meta.icon" class="w-7 h-7" />
      </div>

      <h1 class="text-4xl font-extrabold text-green-900">{{ code }}</h1>
      <p class="text-emerald-700 mt-2 font-medium">{{ resolvedTitle }}</p>
      <p class="text-sm text-emerald-700 mt-1">{{ resolvedMessage }}</p>

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
