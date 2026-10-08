import { useAuthStore } from '~/store/auth';

const PAGE_KEY_MAP = {
    '/': 'dashboard',
    '/employee': 'employee',
    '/shift': 'shift',
    '/report': 'report',
    '/controller': 'controller',
    '/settings': 'settings',
    '/users': 'users',
};

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

        const pageKey = PAGE_KEY_MAP[to.path];
        if (pageKey && !auth.hasPageAccess(pageKey)) {
            return navigateTo("/", { replace: true });
        }
        return;
    }

    // Not logged in -> every page other than /login must go through /login first.
    if (!onLoginPage) return navigateTo("/login", { replace: true });
});
