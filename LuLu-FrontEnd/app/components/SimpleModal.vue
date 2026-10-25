<script>
export class SMData {
  data = ref({
    isShow: false,
    title: '',
    message: '',
    use_ok: false,
    use_cancel: false,
    helper: {
      act: null,
      data: null
    }
  });

  constructor() {
    this.data.value = {
      isShow: false,
      title: '',
      message: '',
      use_ok: false,
      use_cancel: false,
      helper: {
        act: null,
        data: null
      }
    }
  }

  setText(title, message) {
    this.data.value.message = message
    this.data.value.title = title
  }

  showOK() {
    this.data.value.use_ok = true;
    this.data.value.isShow = true
  }

  showCancel() {
    this.data.value.use_cancel = true;
    this.data.value.isShow = true
  }

  showOKCancel() {
    this.data.value.use_cancel = true;
    this.data.value.use_ok = true;
    this.data.value.isShow = true
  }

  isShow() {
    return this.data.value.isShow
  }

  message() {
    return this.data.value.message
  }

  setHelper(act, data) {
    this.data.value.helper.act = act
    this.data.value.helper.data = data
  }

  getHelper(act) {
    if (act == this.data.value.helper.act)
      return this.data.value.helper.data
    else
      return null
  }

  clear() {
    this.data.value = {
      isShow: false,
      title: '',
      message: '',
      use_ok: false,
      use_cancel: false,
      helper: {
        act: '',
        data: ''
      }
    }
  }

  get() { return this.data.value }
}
</script>

<script setup>
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

const
  props = defineProps({
    options: { type: Object, required: true },
    wide: { type: Boolean, default: false },
  }),
  emit = defineEmits(['cancel', 'ok']);

const panel = ref(null)
let previousFocus = null

function clickCancel() {
  emit('cancel')
}

function clickOk() {
  emit('ok')
}

// Keyboard/assistive behaviour the dialog lacked: focus moves into the dialog
// when it opens, Escape cancels, and focus returns to the button that opened it.
// Every delete/confirm flow in the app shares this component, so one place owns it.
watch(() => props.options.isShow, async (open) => {
  if (open) {
    previousFocus = document.activeElement
    await nextTick()
    panel.value?.querySelector('button')?.focus()
  } else if (previousFocus?.focus) {
    previousFocus.focus()
    previousFocus = null
  }
})

const FOCUSABLE = 'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

function onKeydown(event) {
  if (!props.options.isShow) return
  if (event.key === 'Escape') {
    clickCancel()
    return
  }
  // Keep Tab inside the dialog: aria-modal alone does not trap focus.
  if (event.key === 'Tab') {
    const nodes = [...(panel.value?.querySelectorAll(FOCUSABLE) || [])].filter(el => el.offsetParent !== null)
    if (nodes.length === 0) return
    const first = nodes[0]
    const last = nodes[nodes.length - 1]
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault()
      last.focus()
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault()
      first.focus()
    }
  }
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
  <!-- v-if, bukan v-show: v-show gagal override class .block Tailwind
       (display:block menang atas inline display:none) -> backdrop selalu
       menutupi halaman, bahkan form login. Unmount total saat isShow=false. -->
  <div v-if="props.options.isShow"
    class="fixed inset-0 z-[100] grid place-content-center bg-green-950/40 backdrop-blur-sm p-4"
    @click.self="clickCancel">
    <div ref="panel" role="dialog" aria-modal="true" aria-labelledby="modalTitle"
      class="hud-frame w-full max-h-[90vh] overflow-y-auto no-scrollbar rounded-[1.25rem] bg-white p-6 shadow-[0_20px_60px_rgba(5,46,22,.25)] border border-emerald-100"
      :class="props.wide ? 'max-w-4xl sm:min-w-[720px] lg:min-w-[880px]' : 'max-w-xl'">
      <span class="hud-corner hud-corner-tl"></span><span class="hud-corner hud-corner-br"></span>
      <div class="mono text-[10px] tracking-[0.14em] text-emerald-700 mb-1">{{ $t('modal.confirmAction') }}</div>
      <h2 id="modalTitle" class="display text-xl font-extrabold text-green-900 sm:text-2xl">{{ props.options.title }}</h2>

      <div class="mt-4">
        <p class="text-pretty text-emerald-800/80 leading-relaxed">
          {{ props.options.message }}
          <slot />
        </p>
      </div>

      <footer v-if="props.options.use_cancel || props.options.use_ok" class="mt-6 flex justify-end gap-2">
        <button v-show="props.options.use_cancel" v-on:click="clickCancel" type="button"
          class="rounded-xl bg-emerald-50 px-4 py-2 text-sm font-semibold text-emerald-700 border border-emerald-100 hover:bg-emerald-100 transition-colors">
          {{ $t('common.cancel') }}
        </button>

        <button v-show="props.options.use_ok" v-on:click="clickOk" type="button"
          class="rounded-xl bg-emerald-700 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-emerald-800">
          {{ $t('modal.ok') }}
        </button>
      </footer>
    </div>
  </div>
</template>
