// Shared display labels.
//
// The backend returns 'Libur' as a DATA SENTINEL, not as UI copy: h_schedule.rs and
// h_report.rs both build shift names with COALESCE(shifts.shift_name, 'Libur'), and the
// report handler drops any day whose resolved shift is that sentinel. Several components
// also test `shift_name === 'Libur'` to decide a day is off.
//
// So the string must never be renamed at the data layer. Translate it here, at render
// time only, and compare against the raw value in JS.

/** The raw sentinel as it arrives from the API. */
export const OFF_DAY = 'Libur';

/** Label shown to the user for a day with no shift. */
export const OFF_DAY_LABEL = 'Day off';

/**
 * True when a shift name means "no shift on this day".
 * Accepts the sentinel, null/undefined, and empty string.
 */
export function isOffDay(shiftName) {
    return shiftName === OFF_DAY || !shiftName;
}

/** Display name for a shift, translating the off-day sentinel. */
export function shiftLabel(shiftName, fallback = '-') {
    if (isOffDay(shiftName)) return OFF_DAY_LABEL;
    return shiftName || fallback;
}
