<script setup>
useHead(() => ({ status: 404, title: 'Page not found — LuLu System' }))
</script>

<template>
  <ErrorPage :code="404" />
</template>
