<script setup>
const ba = ref(false)

async function sleep() {
  await new Promise(resolve => setTimeout(resolve, 3000));
  ba.value = true;
  return 0;
}
onMounted(() => { sleep(); })
useHead(() => ({
  status: 404,
  title: "Page not found",
}));
</script>

<template>
    404 NOT FOUND
</template>