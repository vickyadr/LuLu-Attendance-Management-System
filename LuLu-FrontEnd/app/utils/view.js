export const useViewUtils = () => {
    
    function setFocusId(id) {
        const inputElement = document.getElementById(id);
        inputElement.focus();
    }

    function setFocusName(name) {
        const inputElement = document.getElementsByName(name);
        inputElement.focus();
    }

    function getFocusElemantId() {
        const focusElement = document.activeElement;
        if (focusElement && focusElement.tagName === 'INPUT' && focusElement.id)
            return focusElement.id;

        return null
    }

    function getFocusElemantName() {
        const focusElement = document.activeElement;
        if (focusElement && focusElement.tagName === 'INPUT' && focusElement.name)
            return focusElement.name;

        return null
    }

    return {
        setFocusId,
        setFocusName,
        getFocusElemantId,
        getFocusElemantName,
    }
}