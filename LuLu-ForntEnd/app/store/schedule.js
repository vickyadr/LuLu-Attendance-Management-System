import { defineStore } from "pinia";
import { useChecker } from "~/utils/checker";

export const useSchedule = defineStore("schedule", () => {
    const contents = ref([]),
        check = useChecker();

    async function addList(data = {
        id: 0,
        name: null,
        shift_name: null,
        dom: 0,
        parrent: 0
    }) {
        contents.value.push(data);
    }

    async function updateList(data = {
        id: 0,
        name: null,
        shift_name: null,
        dom: 0,
        parrent: 0
    }) {
        if (check.isNull(data.id))
            return;

        let index = contents.value.findIndex(x => x.id === data.id);

        if (check.isNull(data.name))
            data.name = contents.value[index].name;
        if (check.isNull(data.shift_name))
            data.shift_name = contents.value[index].shift_name;
        if (check.isNull(data.dom))
            data.dom = contents.value[index].dom;
        if (check.isNull(data.parrent))
            data.parrent = contents.value[index].parrent;

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


    return {
        addList,
        updateList,
        removeList,
        set,
        contents
    };
});