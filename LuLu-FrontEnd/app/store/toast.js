import { defineStore } from "pinia";
import { vGsapVue } from 'v-gsap-nuxt/vue';

export const useToast = defineStore("toast", () => {
    const
        list = ref([])

    async function show(data = {
        message: null,
        type: null,
        time: 0
    }) {
        if (data.time === undefined || data.time === null)
            data.time = 100;
        list.value.unshift(data);
        vGsapVue.to(list.value[0], { time: 0, duration: 3 }).then(() => {
            remove();
        });

    }

    async function remove(index = -1) {
        if (index != -1) {
            list.value.slice(index, 1);
            console.log(`remove ${index}`);
            return;
        }

        for (let i = 0; i < list.value.length; i++) {
            if (list.value[i].time < 1) {
                list.value.splice(i, 1);
            }
        }
    }

    return {
        list,
        show,
        remove
    };
});