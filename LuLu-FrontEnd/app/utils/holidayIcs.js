// Parser for the public Indonesian holiday calendar (.ics).
//
// The calendar is an unofficial endpoint, so everything here is defensive: a
// malformed or partial feed must produce a visible message rather than an
// exception, and the operator reviews the parsed rows before anything is saved.
//
// The feed carries no kind field. Public holidays are tagged in DESCRIPTION as
// "Hari libur nasional"; agreed leave days ("Cuti Bersama") are separated by
// their SUMMARY prefix, which is the only signal the feed offers.
export const HOLIDAY_CALENDAR_URL = '/api/holiday-calendar'

const NATIONAL_DESC = /hari\s+libur\s+nasional/i
const COLLECTIVE_SUMMARY = /^cuti\s+bersama/i

// Unfold RFC 5545 line continuations: a logical line is a line beginning with
// space or tab appended to the previous one. Without this a wrapped DESCRIPTION
// loses its tail and a holiday silently keeps only part of its label.
function unfoldIcs(text) {
    return text.replace(/\r\n/g, '\n').replace(/\n[ \t]/g, '')
}

// Read DTSTART;VALUE=DATE (e.g. 20260101) or DTSTART;TZID=...:20260101T000000
function parseIcsDate(line) {
    const m = line.match(/^DTSTART[^:]*:(\d{4})(\d{2})(\d{2})/)
    return m ? `${m[1]}-${m[2]}-${m[3]}` : null
}

// Read DTSTART for a timed VEVENT (a religious observance). Not written to the
// calendar - these are not days off - but counted so the operator is told what
// was skipped rather than silently losing rows.
function isTimedEvent(icsText) {
    return /^DTSTART[^:]*:\d{8}T\d{6}/m.test(icsText)
}

function unescapeIcs(text) {
    return text
        .replace(/\\n/gi, ' ')
        .replace(/\\,/g, ',')
        .replace(/\\;/g, ';')
        .replace(/\\\\/g, '\\')
        .trim()
}

/**
 * Parse an .ics payload into holiday rows.
 *
 * @returns {{ rows: Array<{holiday_date: string, holiday_name: string, holiday_kind: string}>,
 *            timedObservances: number, skipped: number }}
 *   `rows` is de-duplicated by date, keeping the first entry seen for a date,
 *   because the table is keyed on the date and a second row for the same date
 *   would overwrite the first during the upsert.
 */
export function parseHolidayIcs(text) {
    const unfolded = unfoldIcs(String(text || ''))
    const blocks = unfolded.split('BEGIN:VEVENT').slice(1)

    const byDate = new Map()
    let timedObservances = 0
    let skipped = 0

    for (const block of blocks) {
        const dateMatch = block.match(/^DTSTART[^:]*:(\d{8}(?:T\d{6})?)/m)
        if (!dateMatch) {
            skipped += 1
            continue
        }

        const summaryMatch = block.match(/^SUMMARY[^:]*:(.*)$/m)
        const descMatch = block.match(/^DESCRIPTION[^:]*:(.*)$/m)
        const name = unescapeIcs(summaryMatch ? summaryMatch[1] : '')
        if (!name) {
            skipped += 1
            continue
        }

        // A timed event is a religious observance rather than a day off, so it is
        // counted and skipped rather than saved.
        if (dateMatch[1].includes('T')) {
            timedObservances += 1
            continue
        }

        const date = parseIcsDate(`DTSTART:${dateMatch[1]}`)
        if (!date) {
            skipped += 1
            continue
        }

        const kind = COLLECTIVE_SUMMARY.test(name)
            ? 'collective'
            : NATIONAL_DESC.test(descMatch ? descMatch[1] : '')
              ? 'national'
              : 'office'

        if (!byDate.has(date)) {
            byDate.set(date, { holiday_date: date, holiday_name: name, holiday_kind: kind })
        }
    }

    const rows = [...byDate.values()].sort((a, b) => a.holiday_date.localeCompare(b.holiday_date))
    return { rows, timedObservances, skipped }
}

/**
 * Fetch and parse the public calendar through the app's own proxy.
 *
 * The upstream feed sends no CORS header, so this deliberately targets a
 * same-origin Nitro route rather than the calendar host. Failure reasons are
 * distinguished by the proxy's status so the operator sees the real cause:
 * `timeout`, `fetch` (unreachable), or `empty` (answered, but nothing usable).
 */
export async function fetchHolidayCalendar() {
    let resp
    try {
        resp = await fetch(HOLIDAY_CALENDAR_URL)
    } catch (e) {
        throw new Error('fetch')
    }

    if (resp.status === 504) throw new Error('timeout')
    if (!resp.ok) throw new Error('fetch')

    const text = await resp.text()
    const parsed = parseHolidayIcs(text)
    if (parsed.rows.length === 0) throw new Error('empty')
    return parsed
}
