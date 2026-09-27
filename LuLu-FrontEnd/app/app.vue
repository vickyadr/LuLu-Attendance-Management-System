<script setup lang="ts">
const {
  cookiesEnabled,
  cookiesEnabledIds,
  isConsentGiven,
  isModalActive,
  moduleOptions,
} = useCookieControl()
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
  --color-primary: #059669;
  --color-primary-dark: #047857;
  --color-secondary: #16a34a;
  --color-accent: #84cc16;
  --color-background: #f0fdf4;
  --color-surface: #ffffff;
  --color-text: #14532d;
  --color-muted: #5b9279;
}
html { -webkit-font-smoothing: antialiased; }
body { color: var(--color-text); }
</style>