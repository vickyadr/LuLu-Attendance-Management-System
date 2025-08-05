import { useChecker } from "#imports";

export const useFormater = () => {

    const
        check = useChecker();

    function stripes(index, even, odd){
        if (check.isEven(index))
            return even
        else
            return odd
    }

    function stamp_to_full(date, haveMillis, optTZ){        
        if (check.isNull(haveMillis))
            date = date * 1000;

        if (!check.isNull(optTZ))
            date = date + (optTZ * 3600000);
        
        return new Date(date);
    }

    function stamp_to_naive(date, haveMillis, optTZ){
        if (check.isNull(haveMillis))
            date = date * 1000;

        if (!check.isNull(optTZ))
            date = date + (optTZ * 3600)

        const d = new Date(date);

        const year = d.getFullYear();
        const month = (d.getMonth()+1).toString().padStart(2, '0');
        const day = d.getDate().toString().padStart(2, '0');
        const hours = d.getHours().toString().padStart(2, '0');
        const minutes = d.getMinutes().toString().padStart(2, '0');
        const seconds = d.getSeconds().toString().padStart(2, '0'); 

        return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
    }

    function stamp_to_naive_time(date, haveMillis, optTZ){
        if (check.isNull(haveMillis))
            date = date * 1000;

        if (!check.isNull(optTZ))
            date = date + (optTZ * 3600)

        const d = new Date(date);

        const hours = d.getHours().toString().padStart(2, '0');
        const minutes = d.getMinutes().toString().padStart(2, '0');
        const seconds = d.getSeconds().toString().padStart(2, '0'); 

        return `${hours}:${minutes}:${seconds}`;
    }

    function stamp_to_naive_date(date, haveMillis, optTZ){
        if (check.isNull(haveMillis))
            date = date * 1000;

        if (!check.isNull(optTZ))
            date = date + (optTZ * 3600)

        const d = new Date(date);
        
        const year = d.getFullYear();
        const month = (d.getMonth()+1).toString().padStart(2, '0');
        const day = d.getDate().toString().padStart(2, '0');

        return `${year}-${month}-${day}`;
    }

    function stamp_to_weekday(date, haveMillis, isFull, optTZ){
        if (check.isNull(haveMillis))
            date = date * 1000;

        if (!check.isNull(optTZ))
            date = date + (optTZ * 3600)

        const d = new Date(date);

        return dow(d.getDay()+1, isFull);
    }

    /*function timestamp_format(date, format, optTZ){
        if (!check.isNull(optTZ))
            date = date + (optTZ * 3600)
        const d = new Date(date);

        const year = d.getFullYear();
        const month = (d.getMonth()+1).padStart(2, '0');
        const day = d.getDate().toString().padStart(2, '0');
        const hours = d.getHours().toString().padStart(2, '0');
        const minutes = d.getMinutes().toString().padStart(2, '0');
        const seconds = d.getSeconds().toString().padStart(2, '0'); 

        return '';
    }*/

    function sec_to_hours(input) {
        return Math.floor(input/3600);
    }

    function sec_to_minutes(input) {
        const h = Math.floor(input/3600);
        return Math.floor((input % 3600) / 60)
    }

    function sec_to_naive(input){
        let h = (input < 0) ? Math.floor((86400+input)/3600) : Math.floor(input/3600);
        const m = (input < 0) ? Math.floor(((86400+input) % 3600)/60) : Math.floor((input % 3600) / 60);
        const nextday = h - 24;
        h = (nextday >= 0 )? nextday : h;
        return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}${(nextday >= 0 )?' (+1)':''}${(input < 0 )?' (-1)':''}`;
    }

    function dow(input, isFull) {
        switch (input) {
            case 1: return (isFull) ? "Sunday" : "Sun"
            case 2: return (isFull) ? "Monday" : "Mon"
            case 3: return (isFull) ? "Tuesday" : "Tue"
            case 4: return (isFull) ? "Wednesday" : "Wed"
            case 5: return (isFull) ? "Thursday" : "Thu"
            case 6: return (isFull) ? "Friday" : "Fri"
            case 7: return (isFull) ? "Saturday" : "Sat"
            default: return ""
        }
    }

    return {
        stripes,
        stamp_to_full,
        stamp_to_naive,
        stamp_to_naive_date,
        stamp_to_naive_time,
        stamp_to_weekday,
        sec_to_hours,
        sec_to_minutes,
        sec_to_naive,
        dow
    };
}