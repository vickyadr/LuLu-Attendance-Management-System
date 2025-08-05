import { useAuthStore } from '~/store/auth';

export default defineNuxtRouteMiddleware(async (to, from) => {
    
    useAuthStore().fetchUser(useCookie("XSRF-TOKEN").value).finally(()=>{
        
        if (useAuthStore().isLogIn.valueOf() == true) {   
            if (to.path.replaceAll("/","") == "login")
                return navigateTo("/")       
        }else if (!to.path.replaceAll("/","") != "login")
            return navigateTo("/login")

    }).catch(()=>{
        if (useCookie("XSRF-TOKEN").value === undefined || useCookie("XSRF-TOKEN").value === null)
            return navigateTo("/login")
    })
});