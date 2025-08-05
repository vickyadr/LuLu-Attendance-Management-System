import { defineStore } from "pinia";
import { useChecker } from "~/utils/checker";

export const useTransaction = defineStore("transaction", () => {
    const contents = ref([]),
        check = useChecker();

    async function addList(data = {
        date_time: 0,
        first_name: null,
        last_name: null,
        enroll_type: null,
        location: null,
        enroll_status: 0
    }) {
        contents.value.unshift(data);
    }

    async function updateList(data = {
        date_time: 0,
        first_name: null,
        last_name: null,
        enroll_type: null,
        location: null,
        enroll_status: 0

    }) {
        if (data.id === 0)
            return;

        let index = contents.value.findIndex(x => x.id === data.id);

        if (check.isNull(data.date_time))
            data.date_time = contents.value[index].date_time;
        if (check.isNull(data.first_name))
            data.first_name = contents.value[index].first_name;
        if (check.isNull(data.last_name))
            data.last_name = contents.value[index].last_name;
        if (check.isNull(data.location))
            data.location = contents.value[index].location;
        if (check.isNull(data.enroll_status))
            data.enroll_status = contents.value[index].enroll_status;
        if (check.isNull(data.enroll_type))
            data.enroll_type = contents.value[index].enroll_type;

        contents.value[index] = data;
    }

    async function removeList(id = 0) {
        if (id === 0)
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