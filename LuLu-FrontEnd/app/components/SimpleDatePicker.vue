<template>
  <div class="relative inline-flex">
    <div class="flex pl-3 justify-center px-2 border-y-2 border-l-2 border-purple-100 rounded-l-lg items-center whitespace-nowrap">
        Date :
    </div>
    <div class="flex justify-center border-2 border-purple-100 hover:border-purple-200 items-center">
        <input v-model="inputStartDate" v-on:blur="emitDateRange" v-on:keydown="keyPressed" class="outline-transparent m-1 w-28 text-center" placeholder="YYYY-MM-DD" maxlength="10" type="text"/>
        <FontAwesome :icon="faArrowRight" class="mr-1 h-3"/>
        <input v-model="inputEndDate" v-on:blur="emitDateRange" v-on:keydown="keyPressed" class="outline-transparent m-1 w-28 text-center" placeholder="YYYY-MM-DD" maxlength="10" type="text"/>
    </div>
    <button v-on:click="toggleShow" type="button" class="border-y-2 border-r-2 rounded-r-lg px-2 border-purple-100 hover:border-purple-200">
        <FontAwesome :icon="faCalendarDays"/>
    </button>

  
    <div v-show="isShow" class="absolute end-0 rounded-lg p-2 absolute top-10 z-50 w-[30lvh] overflow-hidden rounded border border-purple-200 bg-white shadow-sm">
        
        <!-- Calendar Navigation -->
        <div class="flex justify-between items-center mb-2 px-2">
            <button
                class="w-8 h-8 flex items-center justify-center bg-gray-100 hover:bg-gray-200 rounded-lg transition-all duration-200 hover:scale-105 text-gray-600"
                @click="previousMonth"
            >
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z" />
                </svg>
            </button>
            <div class="text-base font-semibold text-gray-800">
                {{ currentMonthYear }}
            </div>
            <button
                class="w-8 h-8 flex items-center justify-center bg-gray-100 hover:bg-gray-200 rounded-lg transition-all duration-200 hover:scale-105 text-gray-600"
                @click="nextMonth"
            >
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z" />
                </svg>
            </button>
        </div>

        <!-- Calendar Grid -->
        <div class="">
            <!-- Day Headers -->
            <div class="grid grid-cols-7 gap-0.5 mb-1">
                <div
                class="text-center text-xs font-semibold text-gray-500 py-2"
                v-for="day in dayNames"
                :key="day"
                >
                {{ day }}
                </div>
            </div>

            <!-- Calendar Days -->
            <div class="grid grid-cols-7 gap-0.5">
                <div
                v-for="day in calendarDays"
                :key="day.key"
                :class="getDayClasses(day)"
                @click="selectDate(day)"
                @mouseenter="handleHoverDate(day)"
                @mouseleave="handleClearHover"
                >
                <span class="relative z-10">{{ day.date }}</span>
                </div>
            </div>
        </div>

        <!-- Action buttons -->
        <div class="flex justify-center gap-16 border-t-2 pt-2">
        <button
            class="bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-all duration-200 hover:-translate-y-0.5 disabled:hover:translate-y-0 min-w-14"
            :disabled="!startDate || !endDate"
            @click="applySelection"
        >
            Ok
        </button>
        <button
            class="bg-gray-100 text-gray-700 font-semibold rounded-lg hover:bg-gray-200 border border-gray-300 transition-all duration-200 hover:-translate-y-0.5 min-w-14"
            @click="clearSelection"
        >
            Reset
        </button>
        </div>
    </div>
  </div>
</template>

<script setup>
import { faArrowRight, faCalendarDays } from '@fortawesome/free-solid-svg-icons';
import { useChecker, useFormater } from '#imports';
// Props
const props = defineProps({
  minDate: {
    type: String,
    default: '',
  },
  maxDate: {
    type: String,
    default: '',
  },
  initStartDate: {
    type: String,
    default: '',
  },
  initEndDate: {
    type: String,
    default: '',
  },
  useUtc:{
    type: Boolean,
    default: false,
  }
});
// Imports
const check = useChecker();
const format = useFormater();

// Emits
const emit = defineEmits(['dateRangeSelected', 'dateRangeCleared']);

// Reactive data
const inputStartDate = ref(null);
const inputEndDate = ref(null);
const currentDate = ref(new Date());
const startDate = ref(
  props.initStartDate ? new Date(props.initStartDate) : null
);
const endDate = ref(
  props.initEndDate ? new Date(props.initEndDate) : null
);
const hoverDate = ref(null);
const isSelectingEnd = ref(false);
const isShow = ref(false);

// Constants
const dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const monthNames = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
];

// Computed
const currentMonthYear = computed(() => {
  const month = monthNames[currentDate.value.getMonth()];
  const year = currentDate.value.getFullYear();
  return `${month} ${year}`;
});

const calendarDays = computed(() => {
  const year = currentDate.value.getFullYear();
  const month = currentDate.value.getMonth();

  // First day of month
  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);

  // Days from previous month to fill the grid
  const startDate = new Date(firstDay);
  startDate.setDate(startDate.getDate() - firstDay.getDay());

  // Generate calendar days
  const days = [];
  const currentDay = new Date(startDate);

  // Generate 42 days (6 weeks)
  for (let i = 0; i < 42; i++) {
    const dayData = {
      date: currentDay.getDate(),
      fullDate: new Date(currentDay),
      isCurrentMonth: currentDay.getMonth() === month,
      isToday: isSameDay(currentDay, new Date()),
      isDisabled: isDateDisabled(currentDay),
      key: `${currentDay.getFullYear()}-${currentDay.getMonth()}-${currentDay.getDate()}`,
    };

    if (currentDay.getDate() < 30 && i == 35){
        break;
    }

    days.push(dayData);
    currentDay.setDate(currentDay.getDate() + 1);
  }

  return days;
});

// Methods
const isSameDay = (date1, date2) => {
  return (
    date1.getDate() === date2.getDate() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getFullYear() === date2.getFullYear()
  );
};

const isDateDisabled = (date) => {
  const minDate = props.minDate ? new Date(props.minDate) : null;
  const maxDate = props.maxDate ? new Date(props.maxDate) : null;

  if (minDate && date < minDate) return true;
  if (maxDate && date > maxDate) return true;

  return false;
};

const isDateInRange = (date) => {
  if (!startDate.value || !endDate.value) return false;

  const start = new Date(startDate.value);
  const end = new Date(endDate.value);

  return date >= start && date <= end;
};

const isDateInHoverRange = (date) => {
  if (!startDate.value || !hoverDate.value || endDate.value) return false;

  const start = new Date(startDate.value);
  const hover = new Date(hoverDate.value);
  const min = start <= hover ? start : hover;
  const max = start <= hover ? hover : start;

  return date >= min && date <= max;
};

const getDayClasses = (day) => {
  let classes = [
    'aspect-square',
    'flex',
    'items-center',
    'justify-center',
    'cursor-pointer',
    'rounded-lg',
    'transition-all',
    'duration-200',
    'relative',
    'text-sm',
    'font-medium',
  ];

  // Base states
  if (!day.isCurrentMonth) {
    classes.push('text-gray-300');
  } else {
    classes.push('text-gray-700');
  }

  if (day.isToday) {
    classes.push('font-bold', 'text-blue-600');
  }

  if (day.isDisabled) {
    classes.push('text-gray-300', 'cursor-not-allowed');
  } else {
    classes.push('hover:bg-gray-100', 'hover:scale-105');
  }

  // Selection states
  if (startDate.value && isSameDay(day.fullDate, startDate.value)) {
    classes = classes.filter((c) => !c.includes('hover:bg-gray'));
    classes.push('bg-teal-600', 'text-white', 'font-semibold');

    if (endDate.value && !isSameDay(startDate.value, endDate.value)) {
      classes.push('rounded-r-none');
    }
  } else if (endDate.value && isSameDay(day.fullDate, endDate.value)) {
    classes = classes.filter((c) => !c.includes('hover:bg-gray'));
    classes.push('bg-teal-600', 'text-white', 'font-semibold');

    if (startDate.value && !isSameDay(startDate.value, endDate.value)) {
      classes.push('rounded-l-none');
    }
  } else if (isDateInRange(day.fullDate)) {
    classes = classes.filter((c) => !c.includes('hover:bg-gray'));
    classes.push('bg-teal-100', 'text-teal-800', 'rounded-none');
  } else if (isDateInHoverRange(day.fullDate)) {
    classes = classes.filter((c) => !c.includes('hover:bg-gray'));
    classes.push('bg-teal-50', 'text-teal-700', 'rounded-none');
  }

  if (day.isDisabled) {
    classes = classes.filter(
      (c) => !c.includes('hover:') && !c.includes('cursor-pointer')
    );
    classes.push('cursor-not-allowed');
  }

  return classes.join(' ');
};

const selectDate = (day) => {
  if (day.isDisabled) return;

  const selectedDate = new Date(day.fullDate);

  // both dates are already selected
  if (startDate.value && endDate.value) {
    if (startDate.value > selectedDate){
        startDate.value = selectedDate;
        return;
    }
    if (endDate.value < selectedDate){
        endDate.value = selectedDate;
        return;
    }
    const range = endDate.value - startDate.value;
    const half_range = Math.floor(range/2);
    const new_range = selectedDate - startDate.value;
    if (new_range < half_range && new_range > 0 ){
        startDate.value = selectedDate;
        return;
    }

    if (new_range >= half_range && new_range < range ){
        endDate.value = selectedDate;
        return;
    }

    if (new_range == 0){
        startDate.value = endDate.value;
        endDate.value = null;
        isSelectingEnd.value = true;
        return;
    }

    if (startDate.value > selectedDate && endDate.value > selectedDate){
        endDate.value = selectedDate;
        return;
    }

    endDate.value = null;
    isSelectingEnd.value = true;
    return;
  }

  // First selection
  if (!startDate.value) {
    startDate.value = selectedDate;
    isSelectingEnd.value = true;
    return;
  }

  // Second selection
  if (startDate.value && !endDate.value) {
    if (selectedDate < startDate.value) {
      // If selected date is before start date, swap them
      endDate.value = startDate.value;
      startDate.value = selectedDate;
    } else {
      endDate.value = selectedDate;
    }
    isSelectingEnd.value = false;

    // Emit the selection
    nextTick(() => {
      emitDateRange();
    });
  }
};

const handleHoverDate = (day) => {
  if (day.isDisabled || !isSelectingEnd.value) return;
  hoverDate.value = day.fullDate;
};

const handleClearHover = () => {
  hoverDate.value = null;
};

const previousMonth = () => {
  const newDate = new Date(currentDate.value);
  newDate.setMonth(newDate.getMonth() - 1);
  currentDate.value = newDate;
};

const nextMonth = () => {
  const newDate = new Date(currentDate.value);
  newDate.setMonth(newDate.getMonth() + 1);
  currentDate.value = newDate;
};

const emitDateRange = () => {
  if (!startDate.value || !endDate.value) return;

  const result = {
    startDate: format.stamp_to_naive_date(startDate.value.getTime(), true),//.toISOString().split('T')[0],
    endDate: format.stamp_to_naive_date(endDate.value.getTime(), true),//.toISOString().split('T')[0],
    startTimestamp: getUnixTimestamp(startDate.value),
    endTimestamp: getUnixTimestamp(endDate.value),
    duration: calculateDuration(startDate.value, endDate.value),
  };

  inputStartDate.value = result.startDate;
  inputEndDate.value = result.endDate;

  emit('dateRangeSelected', result);
};

const getUnixTimestamp = (date) => {
  if (!date) return null;
  const tz = (props.useUtc===true ? new Date().getTimezoneOffset()* 60 * -1 : 0);
  return Math.floor(date.getTime() / 1000) + tz;
};

const calculateDuration = (start, end) => {
  if (!start || !end) return 0;
  const timeDiff = end.getTime() - start.getTime();
  return Math.ceil(timeDiff / (1000 * 60 * 60 * 24)) + 1;
};

const keyPressed = (ev) => {
    if (ev.which === 13){
        startDate.value = new Date(inputStartDate.value);
        endDate.value = new Date(inputEndDate.value);
        emitDateRange();
    }
};

const toggleShow = () =>{
    isShow.value = !isShow.value;
    if (isShow){
        startDate.value = new Date(inputStartDate.value);
        endDate.value = new Date(inputEndDate.value);
    }
}

const applySelection = () => {
  if (startDate.value && endDate.value) {
    isShow.value = false;
    emitDateRange();
  }
};

const clearSelection = () => {
  startDate.value = null;
  endDate.value = null;
  hoverDate.value = null;
  isSelectingEnd.value = false;
  currentDate.value = new Date();
  //emit('dateRangeCleared');
};

// Initialize
onMounted(() => {
  if (startDate.value && endDate.value) {
    emitDateRange();
  }
});
</script>
