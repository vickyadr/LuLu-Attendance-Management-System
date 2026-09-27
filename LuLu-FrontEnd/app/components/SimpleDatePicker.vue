<template>
  <div ref="rootEl" class="relative inline-flex items-stretch shrink-0">
    <!-- Trigger -->
    <div class="flex pl-2.5 pr-1 justify-center border-y-2 border-l-2 border-emerald-200 rounded-l-xl bg-emerald-50/60 text-emerald-800 font-semibold items-center whitespace-nowrap mono text-xs tracking-wide">
        Range
    </div>
    <div class="flex justify-center border-y-2 border-emerald-200 bg-white items-center gap-0.5 px-1">
        <input v-model="inputStartDate" @blur="emitDateRange" @keydown="keyPressed" class="outline-none my-1 w-[92px] sm:w-24 text-center mono text-xs sm:text-[13px] py-1 rounded-lg focus:bg-emerald-50" placeholder="YYYY-MM-DD" maxlength="10" type="text"/>
        <FontAwesome :icon="faArrowRight" class="mx-0.5 h-3 text-emerald-700/40 shrink-0"/>
        <input v-model="inputEndDate" @blur="emitDateRange" @keydown="keyPressed" class="outline-none my-1 w-[92px] sm:w-24 text-center mono text-xs sm:text-[13px] py-1 rounded-lg focus:bg-emerald-50" placeholder="YYYY-MM-DD" maxlength="10" type="text"/>
    </div>
    <button @click="toggleShow" type="button" class="border-y-2 border-r-2 rounded-r-xl px-2.5 border-emerald-200 bg-white hover:bg-emerald-50 text-emerald-700 flex items-center justify-center" aria-label="Buka kalender">
        <FontAwesome :icon="faCalendarDays" class="text-[13px]"/>
    </button>

    <!-- Teleported popup: fixed centered on mobile, absolute dropdown on desktop -->
    <Teleport to="body">
      <div v-if="isShow" class="fixed inset-0 z-[60] sm:hidden bg-black/20 backdrop-blur-[1px]" @click="isShow=false" aria-hidden="true"></div>
      <div
        v-show="isShow"
        ref="popupEl"
        :style="popupStyle"
        class="z-[70] bg-white border border-emerald-200 rounded-2xl shadow-[0_16px_40px_rgba(5,46,22,.18)] overflow-hidden
               fixed sm:fixed w-[min(320px,calc(100vw-16px))] sm:w-[300px]
               max-h-[min(440px,78vh)] flex flex-col"
        role="dialog" aria-modal="true"
        @click.stop
      >
        <!-- Header nav -->
        <div class="flex justify-between items-center px-3 pt-3 pb-2 shrink-0">
            <button
                class="w-7 h-7 flex items-center justify-center bg-slate-50 hover:bg-emerald-50 border border-slate-200 hover:border-emerald-200 rounded-lg text-slate-600 hover:text-emerald-700 transition"
                @click="previousMonth" aria-label="Previous month"
            >
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor"><path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/></svg>
            </button>
            <div class="mono text-xs font-bold tracking-wide text-green-900">
                {{ currentMonthYear }}
            </div>
            <button
                class="w-7 h-7 flex items-center justify-center bg-slate-50 hover:bg-emerald-50 border border-slate-200 hover:border-emerald-200 rounded-lg text-slate-600 hover:text-emerald-700 transition"
                @click="nextMonth" aria-label="Next month"
            >
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor"><path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/></svg>
            </button>
        </div>

        <!-- Day headers -->
        <div class="grid grid-cols-7 gap-0.5 px-2 pb-1">
            <div class="text-center mono text-[10px] font-bold tracking-widest text-slate-400 py-1" v-for="day in dayNames" :key="day">{{ day }}</div>
        </div>

        <!-- Days: scrollable if overflow -->
        <div class="grid grid-cols-7 gap-0.5 px-2 pb-2 overflow-auto">
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

        <!-- Footer -->
        <div class="flex items-center gap-2 border-t border-emerald-100 px-3 py-2.5 bg-emerald-50/40 shrink-0">
            <span class="mono text-[10px] tracking-widest text-emerald-700/50 hidden sm:inline">{{ startDate && endDate ? `${durationLabel}` : 'Pick 2 dates' }}</span>
            <span class="flex-1 sm:hidden mono text-[10px] text-emerald-700/60">{{ startDate && endDate ? durationLabel : 'Pick a range' }}</span>
            <button
                class="ml-auto px-3 py-1.5 rounded-full bg-white border border-slate-200 text-slate-700 mono text-xs font-bold hover:bg-slate-50"
                @click="clearSelection"
            >
                Reset
            </button>
            <button
                class="px-4 py-1.5 rounded-full bg-emerald-600 text-white mono text-xs font-bold hover:bg-emerald-700 disabled:opacity-40 disabled:cursor-not-allowed shadow-sm"
                :disabled="!startDate || !endDate"
                @click="applySelection"
            >
                Apply
            </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup>
import { faArrowRight, faCalendarDays } from '@fortawesome/free-solid-svg-icons';
import { useChecker, useFormater } from '#imports';
const props = defineProps({
  minDate: { type: String, default: '' },
  maxDate: { type: String, default: '' },
  initStartDate: { type: String, default: '' },
  initEndDate: { type: String, default: '' },
  useUtc:{ type: Boolean, default: false }
});
const check = useChecker();
const format = useFormater();
const emit = defineEmits(['dateRangeSelected', 'dateRangeCleared']);

const rootEl = ref(null)
const popupEl = ref(null)
const inputStartDate = ref(null);
const inputEndDate = ref(null);
const currentDate = ref(new Date());
const _parseInit = (s)=> s ? (parseLocalYMD(s) || new Date(s)) : null
const startDate = ref(_parseInit(props.initStartDate));
const endDate = ref(_parseInit(props.initEndDate));
const hoverDate = ref(null);
const isSelectingEnd = ref(false);
const isShow = ref(false);
const popupStyle = ref({})

const dayNames = ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'];
const monthNames = ['January','February','March','April','May','June','July','August','September','October','November','December'];

const currentMonthYear = computed(() => {
  const month = monthNames[currentDate.value.getMonth()];
  const year = currentDate.value.getFullYear();
  return `${month} ${year}`;
});
const durationLabel = computed(()=>{
  if (!startDate.value || !endDate.value) return ''
  const d = calculateDuration(startDate.value, endDate.value)
  return `${d} days`
})

const calendarDays = computed(() => {
  const year = currentDate.value.getFullYear();
  const month = currentDate.value.getMonth();
  const firstDay = new Date(year, month, 1);
  const start = new Date(firstDay);
  start.setDate(start.getDate() - firstDay.getDay());
  const days = [];
  const cur = new Date(start);
  for (let i = 0; i < 42; i++) {
    const dayData = {
      date: cur.getDate(),
      fullDate: new Date(cur),
      isCurrentMonth: cur.getMonth() === month,
      isToday: isSameDay(cur, new Date()),
      isDisabled: isDateDisabled(cur),
      key: `${cur.getFullYear()}-${cur.getMonth()}-${cur.getDate()}`,
    };
    if (cur.getDate() < 30 && i == 35){ break; }
    days.push(dayData);
    cur.setDate(cur.getDate() + 1);
  }
  return days;
});

const isSameDay = (date1, date2) => date1.getDate()===date2.getDate() && date1.getMonth()===date2.getMonth() && date1.getFullYear()===date2.getFullYear();
function parseLocalYMD(str){
  if(!str) return null
  const m = String(str).trim().match(/^(\d{4})-(\d{2})-(\d{2})$/)
  if(!m) return null
  const y=+m[1], mo=+m[2], d=+m[3]
  const dt = new Date(y, mo-1, d)
  if(isNaN(dt.getTime())) return null
  return dt
}
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
  let classes = ['h-8','w-8','sm:h-[30px]','sm:w-[30px]','flex','items-center','justify-center','cursor-pointer','rounded-full','transition','relative','text-[13px]','font-medium','mx-auto'];
  if (!day.isCurrentMonth) classes.push('text-slate-300');
  else classes.push('text-slate-700');
  if (day.isToday) classes.push('font-bold','ring-1','ring-emerald-300','text-emerald-700');
  if (day.isDisabled) classes.push('text-slate-300','cursor-not-allowed');
  else classes.push('hover:bg-slate-100');
  if (startDate.value && isSameDay(day.fullDate, startDate.value)) {
    classes = classes.filter((c) => !c.includes('hover:bg-slate'));
    classes.push('!bg-emerald-600','!text-white','font-bold','shadow-sm');
    if (endDate.value && !isSameDay(startDate.value, endDate.value)) classes.push('rounded-r-none');
  } else if (endDate.value && isSameDay(day.fullDate, endDate.value)) {
    classes = classes.filter((c) => !c.includes('hover:bg-slate'));
    classes.push('!bg-emerald-600','!text-white','font-bold','shadow-sm');
    if (startDate.value && !isSameDay(startDate.value, endDate.value)) classes.push('rounded-l-none');
  } else if (isDateInRange(day.fullDate)) {
    classes = classes.filter((c) => !c.includes('hover:bg-slate'));
    classes.push('!bg-emerald-100','!text-emerald-800','rounded-none');
  } else if (isDateInHoverRange(day.fullDate)) {
    classes = classes.filter((c) => !c.includes('hover:bg-slate'));
    classes.push('!bg-emerald-50','!text-emerald-700','rounded-none');
  }
  if (day.isDisabled) {
    classes = classes.filter((c) => !c.includes('hover:') && !c.includes('cursor-pointer'));
    classes.push('cursor-not-allowed');
  }
  return classes.join(' ');
};

const selectDate = (day) => {
  if (day.isDisabled) return;
  const selectedDate = new Date(day.fullDate);
  if (startDate.value && endDate.value) {
    if (startDate.value > selectedDate){ startDate.value = selectedDate; return; }
    if (endDate.value < selectedDate){ endDate.value = selectedDate; return; }
    const range = endDate.value - startDate.value;
    const half_range = Math.floor(range/2);
    const new_range = selectedDate - startDate.value;
    if (new_range < half_range && new_range > 0 ){ startDate.value = selectedDate; return; }
    if (new_range >= half_range && new_range < range ){ endDate.value = selectedDate; return; }
    if (new_range == 0){ startDate.value = endDate.value; endDate.value = null; isSelectingEnd.value = true; return; }
    if (startDate.value > selectedDate && endDate.value > selectedDate){ endDate.value = selectedDate; return; }
    endDate.value = null; isSelectingEnd.value = true; return;
  }
  if (!startDate.value) { startDate.value = selectedDate; isSelectingEnd.value = true; return; }
  if (startDate.value && !endDate.value) {
    if (selectedDate < startDate.value) { endDate.value = startDate.value; startDate.value = selectedDate; }
    else { endDate.value = selectedDate; }
    isSelectingEnd.value = false;
    nextTick(() => { emitDateRange(); });
  }
};
const handleHoverDate = (day) => { if (day.isDisabled || !isSelectingEnd.value) return; hoverDate.value = day.fullDate; };
const handleClearHover = () => { hoverDate.value = null; };
const previousMonth = () => { const nd = new Date(currentDate.value); nd.setMonth(nd.getMonth()-1); currentDate.value = nd; };
const nextMonth = () => { const nd = new Date(currentDate.value); nd.setMonth(nd.getMonth()+1); currentDate.value = nd; };
function syncFromInputs(){
  const a = parseLocalYMD(inputStartDate.value)
  const b = parseLocalYMD(inputEndDate.value)
  if(a) startDate.value = a
  if(b) endDate.value = b
}
const emitDateRange = () => {
  syncFromInputs()
  if (!startDate.value || !endDate.value) return;
  const result = {
    startDate: format.stamp_to_naive_date(startDate.value.getTime(), true),
    endDate: format.stamp_to_naive_date(endDate.value.getTime(), true),
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
        syncFromInputs()
        emitDateRange();
    }
};

function updatePopupPosition(){
  if (!rootEl.value || !isShow.value) return
  const rect = rootEl.value.getBoundingClientRect()
  const vw = window.innerWidth
  const vh = window.innerHeight
  const pw = Math.min(320, vw - 16)
  const ph = 380
  // mobile: centered fixed (CSS handles), skip JS
  if (vw < 640){
    popupStyle.value = {}
    return
  }
  // desktop: try below trigger, flip to above if overflow bottom
  let top = rect.bottom + 8
  let left = rect.right - pw
  // keep inside viewport with 8px margin
  if (left < 8) left = 8
  if (left + pw > vw - 8) left = vw - pw - 8
  if (top + ph > vh - 8){
    const topAbove = rect.top - ph - 8
    if (topAbove >= 8) top = topAbove
    else top = Math.max(8, vh - ph - 8)
  }
  popupStyle.value = { top: top + 'px', left: left + 'px', right: 'auto', bottom: 'auto', transform: 'none' }
}

const toggleShow = () =>{
    isShow.value = !isShow.value;
    if (isShow.value){
        const a = parseLocalYMD(inputStartDate.value)
        const b = parseLocalYMD(inputEndDate.value)
        if(a) startDate.value = a
        if(b) endDate.value = b
        nextTick(()=> updatePopupPosition())
    }
}
const applySelection = () => {
  if (startDate.value && endDate.value) { isShow.value = false; emitDateRange(); }
};
const clearSelection = () => {
  startDate.value = null; endDate.value = null; hoverDate.value = null; isSelectingEnd.value = false; currentDate.value = new Date();
};

function onDocClick(e){
  if (!isShow.value) return
  const t = e.target
  if (rootEl.value && rootEl.value.contains(t)) return
  if (popupEl.value && popupEl.value.contains(t)) return
  // backdrop handles mobile, but desktop click outside closes
  isShow.value = false
}
function onKey(e){ if (e.key === 'Escape' && isShow.value) isShow.value = false }
function onResize(){ if (isShow.value) updatePopupPosition() }

onMounted(() => {
  if (startDate.value && endDate.value) emitDateRange();
  document.addEventListener('click', onDocClick, true)
  document.addEventListener('keydown', onKey)
  window.addEventListener('resize', onResize)
  window.addEventListener('scroll', onResize, true)
});
onBeforeUnmount(()=>{
  document.removeEventListener('click', onDocClick, true)
  document.removeEventListener('keydown', onKey)
  window.removeEventListener('resize', onResize)
  window.removeEventListener('scroll', onResize, true)
})
watch(isShow, (v)=>{ if(v) nextTick(updatePopupPosition) })
</script>
