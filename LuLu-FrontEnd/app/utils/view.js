export const useViewUtils = () => {
    
    function changeFocus(id) {
        const inputElement = document.getElementById(id);
        inputElement.focus();
    }

    return {
        changeFocus,
    }
}