import { useAuthStore } from '~/store/auth';

export default defineNuxtRouteMiddleware(async (to) => {
    const auth = useAuthStore();
    const token = useCookie("XSRF-TOKEN").value;

    if (token) {
        await auth.fetchUser(token);
    } else {
        auth.clearUser();
    }

    const onLoginPage = to.path === "/login" || to.path === "/login/";

    if (auth.isLogIn) {
        if (onLoginPage) return navigateTo("/", { replace: true });
        return;
    }

    if (!onLoginPage) return navigateTo("/login", { replace: true });
});
