<script setup>
import { faExclamationTriangle, faExclamationCircle, faLock, faSearch, faClock, faServer, faWifi, faCreditCard, faTachometerAlt } from '@fortawesome/free-solid-svg-icons'

const error = useError()

const CODE_META = {
  400: { icon: faExclamationTriangle, label: 'BAD REQUEST', title: 'Bad Request', message: 'The request was malformed.' },
  401: { icon: faLock, label: 'UNAUTHORIZED', title: 'Unauthorized', message: 'Your session has expired. Please log in again.' },
  402: { icon: faCreditCard, label: 'PAYMENT REQUIRED', title: 'Payment Required', message: 'Payment is required to access this resource.' },
  403: { icon: faLock, label: 'FORBIDDEN', title: 'Forbidden', message: 'You do not have permission to access this resource.' },
  404: { icon: faSearch, label: 'ROUTE NOT FOUND', title: 'Page Not Found', message: 'The page you are looking for does not exist.' },
  405: { icon: faExclamationCircle, label: 'METHOD NOT ALLOWED', title: 'Method Not Allowed', message: 'This method is not allowed on this resource.' },
  408: { icon: faClock, label: 'REQUEST TIMEOUT', title: 'Request Timeout', message: 'The request took too long to process.' },
  429: { icon: faExclamationCircle, label: 'TOO MANY REQUESTS', title: 'Too Many Requests', message: 'Too many requests. Please slow down.' },
  500: { icon: faServer, label: 'INTERNAL SERVER ERROR', title: 'Internal Server Error', message: 'Something went wrong on our end.' },
  502: { icon: faWifi, label: 'BAD GATEWAY', title: 'Bad Gateway', message: 'The server received an invalid response.' },
  503: { icon: faTachometerAlt, label: 'SERVICE UNAVAILABLE', title: 'Service Unavailable', message: 'The service is temporarily unavailable.' },
  504: { icon: faWifi, label: 'GATEWAY TIMEOUT', title: 'Gateway Timeout', message: 'The server took too long to respond.' },
}
const RETRY_CODES = [408, 429, 500, 502, 503, 504]
const fallback = { icon: faExclamationCircle, label: 'UNKNOWN ERROR', title: 'Unexpected Error', message: 'An unexpected error occurred.' }

const meta = computed(() => CODE_META[error.value?.statusCode] || fallback)
const resolvedTitle = computed(() => error.value?.statusMessage || meta.value.title)
const resolvedMessage = computed(() => error.value?.message || meta.value.message)
const showRetry = computed(() => RETRY_CODES.includes(error.value?.statusCode))

function goBack() { clearError({ redirect: '/' }) }
function retry() { window.location.reload() }
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-[#f6fdf7] tech-grid-soft p-4">
    <div class="hud-frame glass rounded-[2rem] p-10 border border-emerald-100 max-w-md relative overflow-hidden">
      <span class="hud-corner hud-corner-tl"></span>
      <span class="hud-corner hud-corner-br"></span>

      <div class="mono text-[10px] tracking-widest text-emerald-700/40 mb-2">
        ERR {{ error?.statusCode || 500 }} &bull; {{ meta.label }}
      </div>

      <div
        class="mx-auto w-14 h-14 rounded-xl bg-emerald-50 border border-emerald-200 flex items-center justify-center text-emerald-600 mb-4">
        <font-awesome :icon="meta.icon" class="w-7 h-7" />
      </div>

      <h1 class="text-4xl font-extrabold text-green-900">{{ error?.statusCode || 500 }}</h1>
      <p class="text-emerald-700/70 mt-2 font-medium">{{ resolvedTitle }}</p>
      <p class="text-sm text-emerald-700/50 mt-1">{{ resolvedMessage }}</p>

      <div class="mt-6 flex flex-col sm:flex-row items-center justify-center gap-3">
        <button v-if="showRetry" @click="retry" class="btn-emerald px-6 py-2.5 rounded-xl text-sm font-bold">
          Try Again
        </button>
        <button @click="goBack"
          class="px-6 py-2.5 rounded-xl text-sm font-bold border-2 border-emerald-200 text-emerald-700 hover:bg-emerald-50 transition">
          Back to Dashboard
        </button>
      </div>
    </div>
  </div>
</template>
