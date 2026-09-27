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
const
  props = defineProps({
    options: { type: Object, required: true },
  }),
  emit = defineEmits(['cancel', 'ok']);

function clickCancel() {
  emit('cancel')
}

function clickOk() {
  emit('ok')
}

function a() { window.alert('a') }
</script>

<template>

  <div v-if="props.options.isShow"
    class="fixed inset-0 z-[100] grid place-content-center bg-green-950/40 backdrop-blur-sm p-4"
    @click.self="clickCancel">
    <div
      class="hud-frame w-full min-w-md rounded-[1.25rem] bg-white p-6 shadow-[0_20px_60px_rgba(5,46,22,.25)] border border-emerald-100 overflow-hidden">
      <span class="hud-corner hud-corner-tl"></span><span class="hud-corner hud-corner-br"></span>
      <div class="mono text-[10px] tracking-[0.14em] text-emerald-700/40 mb-1">CONFIRM ACTION</div>
      <h2 id="modalTitle" class="display text-xl font-extrabold text-green-900 sm:text-2xl">{{ props.options.title }}</h2>

      <div class="mt-4">
        <p class="text-pretty text-emerald-800/80 leading-relaxed">
          <slot />
        </p>
      </div>

      <footer class="mt-6 flex justify-end gap-2">
        <button
          v-show="(props.options.use_cancel == false && props.options.use_ok == false) || props.options.use_cancel"
          v-on:click="clickCancel" type="button"
          class="rounded-xl bg-emerald-50 px-4 py-2 text-sm font-semibold text-emerald-700 border border-emerald-100 hover:bg-emerald-100 transition-colors">
          Cancel
        </button>

        <button v-show="props.options.use_ok" v-on:click="clickOk" type="button"
          class="rounded bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700">
          Ok
        </button>
      </footer>
    </div>
  </div>
</template>