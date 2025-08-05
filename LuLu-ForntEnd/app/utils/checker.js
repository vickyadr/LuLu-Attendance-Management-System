export const useChecker = () => {
    const config = useRuntimeConfig();

    function isNull(n) {
        if (n === null || n === undefined)
            return true

        if (n.length === 0)
            return true

        if ((n.constructor === Object && Object.keys(n).length === 0))
            return true

        return false
    }

    function isEven(n) {
        return n % 2 == 0;
    }

    function isOdd(n) {
        return Math.abs(n % 2) == 1;
    }
    
    /* 
        NOTE:
        if input data type numeric ordered you can use ["a","b",...] instead
        otherwise {"key1":"val1", "key2":"val2",...}
    */
    function inSwitch(input, keyVal, optDefault) {
        if (!(input === undefined))
            return keyVal[input];
        else if (!(optDefault === undefined))
            return optDefault
        else
            return ""
    }

    return {
        isEven,
        isOdd,
        isNull,
        inSwitch,
    };
};