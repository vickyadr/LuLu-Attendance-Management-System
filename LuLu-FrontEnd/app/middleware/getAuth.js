import { useAuthStore } from '~/store/auth';

// Map route paths to page keys used in user_page_permissions
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

    // Without a token there is no session - skip the API call (it would only 401)
    // and clear stale state: the cookie may already be gone while the store is
    // still true (e.g. after logout), otherwise the guard would let the request through.
    if (token) {
        await auth.fetchUser(token);
    } else {
        auth.clearUser();
    }

    // to.path is exactly "/login". Check the path, not a string "!x != y" that is always true.
    const onLoginPage = to.path === "/login" || to.path === "/login/";

    // auth.isLogIn is already a boolean (Pinia auto-unwraps refs on the store instance).
    if (auth.isLogIn) {
        // Already logged in but still on /login -> redirect to the dashboard.
        if (onLoginPage) return navigateTo("/", { replace: true });

        // Check page-level permissions
        const pageKey = PAGE_KEY_MAP[to.path];
        if (pageKey && !auth.hasPageAccess(pageKey)) {
            return navigateTo("/", { replace: true });
        }
        return;
    }

    // Not logged in -> every page other than /login must go through /login first.
    if (!onLoginPage) return navigateTo("/login", { replace: true });
});
