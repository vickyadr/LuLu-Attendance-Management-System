import { defineStore } from "pinia";
import { useChecker } from "~/utils/checker";

export const useShiftStore = defineStore("shift", () => {
    const contents = ref({}),
        check = useChecker();

    const def_data = {
        id: 0,
        name: null,
        start_time: 0,
        end_time: 0,
        start_enroll: 0,
        end_enroll: 0,
        passday: 0
    };
    
    async function addList(data = def_data) {
        contents.value.push(data);
    }

    async function updateList(data = def_data) {

        if (check.isNull(data.id))
            return;

        let index = contents.value.findIndex(x => x.id === data.id);

        if (check.isNull(data.name))
            data.name = contents.value[index].name;
        if (check.isNull(data.start_time))
            data.start_time = contents.value[index].start_time;
        if (check.isNull(data.end_time))
            data.end_time = contents.value[index].end_time;
        if (check.isNull(data.start_enroll))
            data.start_enroll = contents.value[index].start_enroll;
        if (check.isNull(data.end_enroll))
            data.end_enroll = contents.value[index].end_enroll;
        if (check.isNull(data.passday))
            data.passday = contents.value[index].passday;

        contents.value[index] = data;
    }

    async function removeList(id) {
        if (check.isNull(id))
            return;
        let index = contents.value.findIndex(x => x.id === id);
        contents.value.splice(index, 1);
    }

    function set(data) {
        contents.value = data;
    }

    function get(id){
        let index = contents.value.findIndex(x => x.id === id);
        return contents.value[index];
    }

    return {
        addList,
        updateList,
        removeList,
        set,
        get,
        contents
    };
});