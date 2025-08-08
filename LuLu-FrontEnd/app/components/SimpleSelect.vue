<script>
export class SSData {
    data = ref({
            isShow: false,
            selected: {
                value: '',
                text: '',
            },
            options: [{
                value: 0,
                text: null,
            }],
        });

    constructor() {
        this.data.value = {
            isShow: false,
            selected: {
                value: 0,
                text: '',
            },
            options: [{
                value: 0,
                text: 'None',
            }],
        };
    }

    select(index){
      this.data.value.selected = index
    }

    clear(){
        this.data.value = {
            isShow: false,
            selected: {
                value: '',
                text: '',
            },
            options: [{
                value: '',
                text: '',
            }],
        }
    }
}
</script>

<script setup>
const
    props = defineProps({
        data: {type: SSData, required: true},
    }),
    emit = defineEmits(['change', 'click']);

function setHide(){
    props.data.data.value.isShow = false
}

function changeSelection(index){
    props.data.data.value.selected.value = props.data.data.value.options[0].value
    props.data.data.value.selected.text =  props.data.data.value.options[index].text
    
    setHide()
}
</script>

<template>

  <div class="relative inline-flex">
    <span
      class="inline-flex divide-x divide-gray-300 overflow-hidden rounded border border-gray-300 bg-white shadow-sm"
    >
      <button
        v-on:click="data.data.value.isShow = !data.data.value.isShow"
        type="button"
        class="px-3 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 hover:text-gray-900 focus:relative"
      >
        {{ data.data.value.selected.text }}
      </button>
  
      <button
        v-on:click="data.data.value.isShow = !data.data.value.isShow"
        type="button"
        class="px-3 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 hover:text-gray-900 focus:relative"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
          class="size-4"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
        </svg>
      </button>
    </span>
  
    <div
      v-show="data.data.value.isShow"
      class="absolute start-0 top-12 z-auto w-56 divide-y divide-gray-200 overflow-hidden rounded border border-gray-300 bg-white shadow-sm"
    >
      <div>
        <a v-for="(d, index) in data.data.value.options"
          v-on:click="changeSelection(index)"
          class="block px-3 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 hover:text-gray-900"
        >
          {{ d.text }}
        </a>
      </div>
      <button
        v-on:click="changeSelection(0)"
        type="button"
        class="block w-full px-3 py-2 text-left text-sm font-medium text-red-700 transition-colors hover:bg-red-50"
      >
        Delete
      </button>
    </div>
  </div>
</template>