// Proxy for the public Indonesian holiday calendar.
//
// The calendar feed sends no `Access-Control-Allow-Origin` header, so the
// browser cannot fetch it directly. This route fetches it server-side and
// streams the raw body back with a text content type, which lifts the CORS
// problem without storing anything: the response is still parsed and reviewed
// in the browser before any row is written.
//
// Upstream failures surface as explicit HTTP statuses (502 = the calendar did
// not answer or answered with an error, 504 = it took too long) rather than
// an empty 200, so the panel can show a real reason instead of pretending the
// calendar is empty.
const CALENDAR_URL =
    'https://calendar.google.com/calendar/ical/id.indonesian%23holiday%40group.v.calendar.google.com/public/basic.ics'

export default defineEventHandler(async (event) => {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), 25_000)

    try {
        const upstream = await fetch(CALENDAR_URL, {
            signal: controller.signal,
            redirect: 'follow',
            headers: { 'User-Agent': 'LuLu-Attendance/1.0 (holiday-calendar-proxy)' },
        })

        if (!upstream.ok) {
            throw createError({
                statusCode: 502,
                statusMessage: `calendar answered ${upstream.status}`,
            })
        }

        const body = await upstream.text()

        // Trust the parse, not the label: an upstream outage page is HTML with
        // a 200 status. The preview parser rejects non-ICS content, but refuse
        // it here already so the client never mistakes it for an empty year.
        if (!body.includes('BEGIN:VCALENDAR')) {
            throw createError({
                statusCode: 502,
                statusMessage: 'calendar did not return a calendar file',
            })
        }

        setHeader(event, 'content-type', 'text/calendar; charset=utf-8')
        return body
    } catch (e) {
        if (e?.statusCode === 502) throw e
        throw createError({
            statusCode: e?.name === 'AbortError' ? 504 : 502,
            statusMessage: e?.name === 'AbortError' ? 'calendar timed out' : 'calendar unreachable',
        })
    } finally {
        clearTimeout(timer)
    }
})
