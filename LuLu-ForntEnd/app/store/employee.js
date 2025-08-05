import { defineStore } from "pinia";
import { useChecker } from "~/utils/checker";

// Im lazy, its just a copy paste from device.js with no changes
export const useEmployee = defineStore("employee", () => {
    const contents = ref([]),
        check = useChecker();

    async function addList(data = {
        id:0,
        name: '',
        sn: '',
        h_id:0,
        location:'',
        handler: 0,
        timezone: 0,
    }) {
        contents.value.push(data);
    }

    async function updateList(data = {
        id:0,
        name: '',
        sn: '',
        h_id:0,
        location:'',
        handler: 0,
        timezone: 0,
    }) {
        if (check.isNull(data.id))
            return;

        let index = contents.value.findIndex(x => x.id === data.id);

        if (check.isNull(data.name))
            data.name = contents.value[index].name;
        if (check.isNull(data.sn))
            data.sn = contents.value[index].sn;
        if (check.isNull(data.location))
            data.location = contents.value[index].location;
        if (check.isNull(data.handler))
            data.handler = contents.value[index].handler;
        if (check.isNull(data.h_id))
            data.h_id = contents.value[index].h_id;
        if (check.isNull(data.timezone))
            data.timezone = contents.value[index].timezone;

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