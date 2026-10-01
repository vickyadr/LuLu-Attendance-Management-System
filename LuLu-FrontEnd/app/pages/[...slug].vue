<script setup>
const route = useRoute()
const slug = route.params.slug
const firstSegment = Array.isArray(slug) ? slug[0] : slug
const code = /^\d{3}$/.test(firstSegment) ? parseInt(firstSegment, 10) : 404

useHead(() => ({ status: code, title: `${code} — LuLu System` }))
</script>

<template>
  <ErrorPage :code="code" />
</template>
