import { defineStore } from "pinia";
import { useChecker } from "~/utils/checker";

export const useSchedule = defineStore("schedule", () => {
    const contents = ref([]),
        check = useChecker();

    function addList(data = {
        id: 0,
        name: null,
        shift_name: null,
        dom: 0,
        parrent: 0
    }) {
        contents.value.push(data);
    }

    function updateList(data = {
        id: 0,
        name: null,
        shift_name: null,
        dom: 0,
        parrent: 0
    }) {
        if (check.isNull(data.id))
            return;

        const index = contents.value.findIndex(x => x.id === data.id);
        if (index === -1) return;

        const existingItem = contents.value[index];
        contents.value[index] = {
            ...existingItem,
            ...data,
            name: data.name ?? existingItem.name,
            shift_name: data.shift_name ?? existingItem.shift_name,
            dom: data.dom ?? existingItem.dom,
            parrent: data.parrent ?? existingItem.parrent
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