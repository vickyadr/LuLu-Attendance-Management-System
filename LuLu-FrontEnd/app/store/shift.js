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
    
    function addList(data = def_data) {
        contents.value.push(data);
    }

    function updateList(data = def_data) {
        if (check.isNull(data.id))
            return;

        const index = contents.value.findIndex(x => x.id === data.id);
        if (index === -1) return;

        const existingItem = contents.value[index];
        contents.value[index] = {
            ...existingItem,
            ...data,
            name: data.name ?? existingItem.name,
            start_time: data.start_time ?? existingItem.start_time,
            end_time: data.end_time ?? existingItem.end_time,
            start_enroll: data.start_enroll ?? existingItem.start_enroll,
            end_enroll: data.end_enroll ?? existingItem.end_enroll,
            passday: data.passday ?? existingItem.passday
        };
    }

    function removeList(id) {
        if (check.isNull(id))
            return;
        const index = contents.value.findIndex(x => x.id === id);
        if (index !== -1) {
            contents.value.splice(index, 1);
        }
    }

    function set(data) {
        contents.value = data;
    }

    function get(id) {
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