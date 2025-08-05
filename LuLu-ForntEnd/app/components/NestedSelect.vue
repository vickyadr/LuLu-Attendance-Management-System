<script setup>
//import { useChecker, useFormater } from '#imports';

const
    check = useChecker(),
    //format = useFormater(),
    selected = ref(null),
    emit = defineEmits(['update']),
    props = defineProps({
        def: { type: Object, required: false, default:{key:null, val:null} },
        data: { type: Object, required:true },
        keyId: { type: Number, required: true },
    });

function updateValue(){
    emit('update', {key: props.keyId, val:selected.value})
}

defineExpose({
    selected
})
</script>

<template>
    <select v-model="selected" v-on:change="updateValue">
        <option v-if="!check.isNull(def)" :value="props.def.val" default>{{ props.def.key }}</option>
        <option v-for="o in props.data" :value="o.id">{{ o.name }}</option>
    </select>
</template>