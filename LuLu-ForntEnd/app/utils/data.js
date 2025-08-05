/*export const useManipulator = () => {

    function sortCtx(contents, sortType) {

        if (sortType == null)
            return contents

        if (sortType == 1) // 1 = A-Z
            return contents.sort((a, b) => (a.c_name < b.c_name) ? -1 : (a.c_name > b.c_name) ? 1 : 0)
        else if (sortType == 2) // 2 = FIFO
            return contents.sort((a, b) => a.c_number - b.c_number)
        else if (sortType == 3) // 3 = LIFO
            return contents.sort((a, b) => b.c_number - a.c_number)
        else if (sortType == 0) // 4 = Z-A
            return contents.sort((a, b) => (a.c_name < b.c_name) ? 1 : (a.c_name > b.c_name) ? -1 : 0)
        else
            return contents

    }

    return {
        sortCtx,
    };    
}*/