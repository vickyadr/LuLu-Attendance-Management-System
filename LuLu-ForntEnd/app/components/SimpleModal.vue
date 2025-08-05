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

    setText(title, message){
      this.data.value.message = message
      this.data.value.title = title
    }

    showOK(){
      this.data.value.use_ok = true;
      this.data.value.isShow = true
    }

    showCancel(){
      this.data.value.use_cancel = true;
      this.data.value.isShow = true
    }

    showOKCancel(){
      this.data.value.use_cancel = true;
      this.data.value.use_ok = true;
      this.data.value.isShow = true
    }

    isShow(){
        return this.data.value.isShow
    }

    message(){
        return this.data.value.message
    }

    setHelper(act, data){
      this.data.value.helper.act = act
      this.data.value.helper.data = data
    }

    getHelper(act){
      if (act == this.data.value.helper.act)
        return this.data.value.helper.data
      else
        return null
    }

    clear(){
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

    get(){ return this.data.value }
}
</script>

<script setup>
const
    props = defineProps({
        options: {type: Object, required: true},
    }),
    emit = defineEmits(['cancel', 'ok']);

function clickCancel() {
    emit('cancel')
}

function clickOk() {
    emit('ok')
}
</script>

<template>
  <div
    class="fixed inset-0 z-50 grid place-content-center bg-black/50 p-4 block">
    <div class="w-full min-w-[50lvh] rounded-lg bg-white p-6 shadow-lg">
      <h2 id="modalTitle" class="text-xl font-bold text-gray-900 sm:text-2xl underline">{{props.options.title}}</h2>
  
      <div class="mt-4">
        <p class="text-pretty text-gray-700">
          <slot/>
        </p>
      </div>
  
      <footer class="mt-6 flex justify-end gap-2">
        <button
          v-show="(props.options.use_cancel == false && props.options.use_ok == false) || props.options.use_cancel"
          v-on:click="clickCancel"
          type="button"
          class="rounded bg-gray-100 px-4 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-200"
        >
          Cancel
        </button>
  
        <button
          v-show="props.options.use_ok"
          v-on:click="clickOk"
          type="button"
          class="rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700"
        >
          Ok
        </button>
      </footer>
    </div>
  </div>
</template>