<script setup lang="ts">
const {
  cookiesEnabled,
  cookiesEnabledIds,
  isConsentGiven,
  isModalActive,
  moduleOptions,
} = useCookieControl()

// example: react to a cookie being accepted
watch(
  () => cookiesEnabledIds.value,
  (current, previous) => {
    if (
      !previous?.includes('XSRF-TOKEN') &&
      current?.includes('XSRF-TOKEN')
    ) {
      window.location.reload()
    }
  },
  { deep: true },
)
</script>

<template>
  <div>
    <CookieControl locale="en" />
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </div>
</template>

<style>
:root {
  --color-primary: #000000;
  --color-secondary: #000000;
  --color-accent: #000000;
  --color-background: #000000;
  --color-text: #000000;
}
</style>
