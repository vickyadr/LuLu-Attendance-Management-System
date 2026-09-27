// Once again, im too lazy to rewrite DB and redebug so i create it instead
// Ehe...
export const OFF_DAY = 'Libur';

export const OFF_DAY_LABEL = 'Day off';

export function isOffDay(shiftName) {
    return shiftName === OFF_DAY || !shiftName;
}

export function shiftLabel(shiftName, fallback = '—') {
    if (isOffDay(shiftName)) return OFF_DAY_LABEL;
    return shiftName || fallback;
}