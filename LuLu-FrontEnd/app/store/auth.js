import { defineStore } from "pinia";

export const useAuthStore = defineStore("auth", () => {
    const token = useCookie("XSRF-TOKEN");
    const config = useRuntimeConfig();
    const user = {
        fname: ref(null),
        lname: ref(null),
        level: ref(0),
    };

    const isLogIn = computed(() => !!user.fname.value && !!user.lname.value && !!user.level.value);
    
    const displayName = computed(() => {
        const s = `${user.fname.value || ''} ${user.lname.value || ''}`.trim();
        return s || 'Guest';
    });
    const initials = computed(() =>
        (displayName.value || 'GG').split(/\s+/).slice(0, 2).map(p => p[0]).join('').toUpperCase()
    );
    const level = computed(() => user.level.value || 0);

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

        await $fetch(`${config.public.apiBase}/login`,
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
            await $fetch(`${config.public.apiBase}/login/user`, {
                method: "GET",
                headers: confHeaders(),
            }).then((response) => {
                if (response.code == 200) {
                    user.fname.value = response.data[0].user_fname;
                    user.lname.value = response.data[0].user_lname;
                    user.level.value = response.data[0].user_level;
                } else {
                    // Token ditolak server -> sesi tidak valid, reset state.
                    clearUser();
                }
            });
        } catch (err) {
            clearUser();
        }
    }

    function clearUser() {
        user.fname.value = null;
        user.lname.value = null;
        user.level.value = 0;
    }

    async function logout() {
        await $fetch(`${config.public.apiBase}/login/logout`, {
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
        level,
    };
});