<script>
export class SDPData {
    data = ref({
            isShow: false,
            selected: {
                value: '',
                text: '',
            },
            options: [{
                value: 0,
                text: null,
            }],
        });

    constructor() {
        this.data.value = {
            isShow: false,
            selected: {
                value: 0,
                text: '',
            },
            options: [{
                value: 0,
                text: 'None',
            }],
        };
    }

    select(index){
      this.data.value.selected = index
    }

    clear(){
        this.data.value = {
            isShow: false,
            selected: {
                value: '',
                text: '',
            },
            options: [{
                value: '',
                text: '',
            }],
        }
    }
}
</script>

<script setup>
import { faArrowLeft, faArrowRight, faCalendarDays } from '@fortawesome/free-solid-svg-icons';
import { useChecker, useFormater } from '#imports';

const
    props = defineProps({
        data: {type: SDPData, required: true},
    }),
    check = useChecker(),
    format = useFormater(),
    calendar = ref({
        year: 1970,
        month: 1,
        date: 1,
    }),
    range = reactive({
        start: {
            year:null,
            month:null,
            date:null,
            timestamp:0,
        },
        end: {
            year:null,
            month:null,
            date:null,
            timestamp:0,
        }
    }), 
    emit = defineEmits(['change', 'click']);

function setHide(){
    props.data.data.value.isShow = false
}

function setShow(){
    if (props.data.data.value.isShow == true)
        return setHide()
    
    const d = new Date()
    calendar.value.year = d.getFullYear()
    calendar.value.month = d.getMonth()+1
    calendar.value.date = d.getDate()
    props.data.data.value.isShow = true
}

function parseDate(year, month){
    const total = new Date(year, month, 0).getDate()
    const parse_yymm = year +"-"+month+"-";
    const total_prev = new Date((month > 1) ? year : year-1, (month > 1) ? month-1 : 12, 0).getDate()
    const start_d = new Date(parse_yymm+1).getDay()
    const end_d = new Date(parse_yymm+total).getDay()
    const to = total + ((end_d != 6) ? 6 - end_d : 0)

    let i = 1 - start_d,
        date = [],
        j = 0
    while (i <= to){
        date[j] = i

        if (i < 1)
            date[j] = total_prev + i

        if (i > total)
            date[j] = i - total
        i++
        j++
    }

    return date
    
}

function selectDate(date){
    const timestamp = format.define_timestamp(calendar.value.year, calendar.value.month, date)

    if (range.start.timestamp === timestamp && range.end.timestamp === timestamp){
        range.start.year = range.end.year = null
        range.start.month = range.end.month = null
        range.start.date = range.end.date = null
        range.start.timestamp = range.end.timestamp = 0
        return;
    }

    if (range.start.timestamp === 0 && range.end.timestamp === 0){
        range.end.year = calendar.value.year
        range.end.month = calendar.value.month
        range.end.date = calendar.value.date = date
        range.end.timestamp = timestamp

        range.start.year = calendar.value.year
        range.start.month = calendar.value.month
        range.start.date = calendar.value.date = date
        range.start.timestamp = timestamp
        return;
    }

    if (range.start.timestamp > timestamp) {
        range.start.year = calendar.value.year
        range.start.month = calendar.value.month
        range.start.date = calendar.value.date = date
        range.start.timestamp = timestamp
        return;
    }
    
    if (timestamp == range.start.timestamp){
        range.start.year = range.end.year
        range.start.month = range.end.month
        range.start.date = range.end.date
        range.start.timestamp = range.end.timestamp
        return;
    }

    if (timestamp == range.end.timestamp) {
        range.end.year = range.start.year
        range.end.month = range.start.month
        range.end.date = range.start.date
        range.end.timestamp = range.start.timestamp
        return;
    }

    if (range.start.timestamp < timestamp) {
        range.end.year = calendar.value.year
        range.end.month = calendar.value.month
        range.end.date = calendar.value.date = date
        range.end.timestamp = timestamp
        return;
    }

}

function parseMonth(month){
    switch (month) {
        case 1:
            return "January"
        case 2:
            return "February"
        case 3:
            return "March"
        case 4:
            return "April"
        case 5:
            return "May"
        case 6:
            return "June"
        case 7:
            return "July"
        case 8:
            return "August"
        case 9:
            return "September"
        case 10:
            return "October"
        case 11:
            return "November"
        case 12:
            return "December"
        default:
            return "Unknown"
    }
}

function isInRange(date, index){
    
    if (date > index + 1 || index - date > 25)
        return false;

    if (check.isNull(range.start.date) || check.isNull(range.end.date))
        return false;

    if (!(calendar.value.year >= range.start.year && calendar.value.year <= range.end.year))
        return false;

    if (!(calendar.value.month >= range.start.month && calendar.value.month <= range.end.month))
        return false;
 
    if (!(date >= range.start.date && date <= range.end.date))
        return false;

    return true;
}
</script>

<template>

  <div class="relative inline-flex">
    <span
      class="inline-flex divide-x divide-gray-300 overflow-hidden rounded border border-gray-300 bg-white shadow-sm"
    >
      <input type="date"/>
  
      <button
        v-on:click="setShow()"
        type="button"
        class="px-3 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 hover:text-gray-900 focus:relative"
      >
        <FontAwesome :icon="faCalendarDays"/>
      </button>
    </span>
  
    <div
      v-show="data.data.value.isShow"
      class="rounded-lg p-2 absolute start-0 top-12 z-auto w-[30lvh] overflow-hidden rounded border border-purple-200 bg-white shadow-sm"
    >
      <div>
        <div class="flex justify-between" id="head">
            <button type="button" v-on:click="()=>{if (calendar.month > 1) { calendar.month -- }else{ calendar.month = 12; calendar.year --; }}">
                <FontAwesome :icon="faArrowLeft" class="text-teal-600"/>
            </button>
            <span class="font-medium">
                {{ parseMonth(calendar.month) }} {{ calendar.year }}
            </span>
            <button type="button" v-on:click="()=>{if (calendar.month < 12) { calendar.month ++ }else{ calendar.month = 1; calendar.year ++; }}">
                <FontAwesome :icon="faArrowRight" class="text-teal-600"/>
            </button>
        </div>
        <div class="grid grid-cols-7 text-center">
            <div class="text-gray-400 m-2">Su</div>
            <div class="text-gray-400 m-2">Mo</div>
            <div class="text-gray-400 m-2">Tu</div>
            <div class="text-gray-400 m-2">We</div>
            <div class="text-gray-400 m-2">Th</div>
            <div class="text-gray-400 m-2">Fr</div>
            <div class="text-gray-400 m-2">Sa</div>

            <button v-for="(i, index) in parseDate(calendar.year, calendar.month)" v-on:click="selectDate(i)"
                :class="(isInRange(i, index))? 'bg-teal-600':''" class="px-2 my-1">
                <span :class="(i > index + 1 || index - i > 25) ? 'text-gray-300':''">{{ i }}</span>
            </button>

        </div>
      </div>
    </div>
  </div>
</template>