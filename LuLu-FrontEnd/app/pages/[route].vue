<script setup>
const router = useRouter();
const { path } = useRoute();

const { routes } = await useAsyncData('error_page', () => {
  // Replace this "false" your own logic to verify if the path is valid
  const isPathValid = (path) => false;
  if (!isPathValid(path)) {
    router.push({ path: "/404" });
  }
  
})

</script>