import { defineStore } from "pinia";
import { useNotifyStore } from "~/store/notify";

export const useAuthStore = defineStore("auth", () => {
    const token = useCookie("XSRF-TOKEN");
    const config = useRuntimeConfig();
    const user = {
        nickname: ref(null),
        fname: ref(null),
        lname: ref(null),
    };

    const permissions = ref({});

    const isLogIn = computed(() => !!user.fname.value && !!user.lname.value);

    const displayName = computed(() => {
        const s = `${user.fname.value || ''} ${user.lname.value || ''}`.trim();
        return s || 'Guest';
    });
    const initials = computed(() =>
        (displayName.value || 'GG').split(/\s+/).slice(0, 2).map(p => p[0]).join('').toUpperCase()
    );

    const isAdmin = computed(() => hasPageAccess('users'));

    function permFor(pageKey) {
        return permissions.value[pageKey] || null;
    }

    function hasPageAccess(pageKey) {
        return permFor(pageKey)?.can_view === true;
    }

    function hasActionAccess(pageKey, action) {
        const perm = permFor(pageKey);
        if (!perm) return false;
        if (action === 'view') return perm.can_view === true;
        return perm[action] === true;
    }

    const confHeaders = () => {
        // FIXME: Temporary Fix, server auth need some optimization
        let tmp_token = token.value;
        if (token.value === undefined) { tmp_token = "emp"; }

        return {
            "Content-Type": "application/json",
            Authorization: `Bearer ${tmp_token}`,
        }
    };

    async function attemptLogin(data) {
        let resp = null;

        await apiFetch(`${config.public.apiBase}/login`,
            {
                body: data,
                method: "POST",
                headers: confHeaders(),

            }).then(async (response) => {
                if (response.code == 200) {
                    useCookie("XSRF-TOKEN", {
                        //httpOnly: true,
                        //expires: Date.now(),
                        //sameSite: "strict",
                    }).value = response.data[0].token;
                    await useAuthStore().fetchUser(response.token);
                }
                resp = response;
            });
        return resp;
    }

    async function fetchUser(token) {
        try {
            await apiFetch(`${config.public.apiBase}/login/user`, {
                method: "GET",
                headers: confHeaders(),
            }).then(async (response) => {
                if (response.code == 200) {
                    user.nickname.value = response.data[0].user_nickname;
                    user.fname.value = response.data[0].user_fname;
                    user.lname.value = response.data[0].user_lname;
                    await fetchPermissions();
                } else {
                    // Token rejected by the server -> session is invalid, reset state.
                    clearUser();
                }
            });
        } catch (err) {
            // Error = 400/401/token expired. This used to be empty here, so the
            // old state stayed active and isLogIn stayed true even with a dead token.
            clearUser();
        }
    }

    function clearUser() {
        user.nickname.value = null;
        user.fname.value = null;
        user.lname.value = null;
        permissions.value = {};
    }

    async function fetchPermissions() {
        try {
            const resp = await apiFetch(`${config.public.apiBase}/user/permissions`, {
                method: 'GET',
                headers: confHeaders(),
            });
            if (resp.code === 200 && Array.isArray(resp.data)) {
                const map = {};
                for (const row of resp.data) {

                    map[row.page_key] = {
                        can_view: row.can_view === true,
                        can_add: row.can_add === true,
                        can_edit: row.can_edit === true,
                        can_delete: row.can_delete === true,
                    };
                }
                permissions.value = map;
            }
        } catch (e) {
            console.warn('[auth] fetchPermissions failed', e);
        }
    }

    async function logout() {
        await apiFetch(`${config.public.apiBase}/login/logout`, {
            method: "GET",
            headers: confHeaders(),
        }).then((response) => {
            if (response.code==200) {
                useCookie("XSRF-TOKEN", {
                    maxAge: 5,
                    //expires: Date.now()
                }).value = null

                clearUser();
                token.value = null

                try {
                    const notify = useNotifyStore();
                    notify.reset();
                } catch {
                    // Store unavailable (e.g. during SSR) — nothing to clear.
                }
                window.location.reload()
                //navigateTo("/login", { replace: true })
            }
        })
    }


    return {
        attemptLogin,
        fetchUser,
        clearUser,
        logout,
        user,
        isLogIn,
        confHeaders,
        displayName,
        initials,
        isAdmin,
        hasPageAccess,
        hasActionAccess,
        permissions,
        fetchPermissions,
    };
});
