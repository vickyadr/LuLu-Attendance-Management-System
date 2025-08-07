import { defineStore } from "pinia";
import { useChecker } from "~/utils/checker";

// Im lazy, its just a copy paste from device.js with no changes
export const useReportStore = defineStore("report", () => {
    const contents = ref([]),
        check = useChecker(),
        data_def = {
            id: 0,
            first_name: null,
            last_name: null,
            departement: null,
            enroll_dow: 0,
            date: 0,
            tz: 0,
            enroll_device: null,
            enroll_location: null,
            shift_start: null,
            shift_end: null,
            shift_name: null,
            schedule_name: null,
            start: null,
            end: null,
            working_time: null,
            late_time: null
    };

    async function addList(data = data_def) {
        contents.value.push(data);
    }

    async function updateList(data = data_def) {
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