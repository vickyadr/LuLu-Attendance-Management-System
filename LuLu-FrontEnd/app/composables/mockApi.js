// ── Mock API for the public demo build ──────────────────────────────────
//
// The demo is published to a static host (no server process, no Postgres), so
// the API has to run inside the browser. This module answers the exact routes the
// app calls, reproducing the Actix backend's contracts one-for-one:
//
//   - every response is `{ message, data, code }` (utility/stor.rs)
//   - list endpoints return `data` as an ARRAY (GenericResponse::ok/single);
//     NoDataResponse endpoints return an empty array
//   - login returns `data: [{ user_id: "<string>", token }]` (models/m_login.rs)
//   - /login/user returns `data: [{ user_nickname, user_fname, user_lname }]`
//   - /transaction/live returns `data` newest-first and puts the newest
//     `enroll_id` in `message` - LiveTable.vue parses it for its "last seen" stamp
//   - /user/list returns `UserWithPermissions`: `permissions` is a MAP keyed by
//     page_key, never an array (models/m_user.rs)
//   - field renames match the serde attributes (employee `id`/`first_name`,
//     device `sn`/`h_id`, schedule `type`/`dom`/`parrent`, …)
//   - success/failure messages are the backend's own strings, because the UI
//     toasts `response.message` verbatim and users.vue matches one of them
//
// It is wired in at the single choke point every page already goes through - the
// `apiFetch` composable - so no page, store or component has to know whether it is
// talking to the real backend or the mock.
//
// Enable with NUXT_PUBLIC_DEMO_MODE=true. With the flag off, `apiFetch` passes
// through to the real backend untouched, so dev and production are unaffected.
//
// Attendance is generated deterministically per (employee, day) - seeded from the
// ids, never Math.random - so a reload shows the same attendance for the same day
// instead of reshuffling on every refresh.

/** Page keys, matching the `users` page grid and the route middleware. */
const PAGE_KEYS = ['dashboard', 'employee', 'shift', 'report', 'controller', 'settings', 'users']

/** Day-off sentinel. The backend builds it with COALESCE and drops those rows. */
const OFF_DAY = 'Libur'

/** Tolerance ceiling, from handler/h_settings.rs (MAX_TOL). */
const MAX_TOL = 3600

/** Demo accounts. `permissions` maps page_key -> may this account open the page. */
const DEMO_USERS = [
    {
        user_id: 1,
        user_nickname: 'demo',
        user_fname: 'DEMO',
        user_lname: 'USER',
        password: 'demo12345',
        permissions: { dashboard: 1, employee: 1, shift: 1, report: 1, controller: 1, settings: 1, users: 1 },
    },
    {
        user_id: 2,
        user_nickname: 'auditor',
        user_fname: 'AUDIT',
        user_lname: 'VIEWER',
        password: 'audit12345',
        permissions: { dashboard: 1, employee: 1, shift: 1, report: 1, controller: 1, settings: 0, users: 0 },
    },
    {
        user_id: 3,
        user_nickname: 'staf',
        user_fname: 'STAFF',
        user_lname: 'ADMIN',
        password: 'staf12345',
        permissions: { dashboard: 1, employee: 1, shift: 0, report: 1, controller: 0, settings: 0, users: 0 },
    },
]

/**
 * Attendance machines. `status` is device_state: 1 = online, 2 = offline,
 * 3 = disabled. `online` (device_online) is the last-seen epoch, offset here so
 * the list renders the same relative age on every load.
 */
const DEVICES = [
    { id: 1, name: 'Floor 2 Attendance Machine', sn: 'ASEE665427309', h_id: 4, handler: 'iClock / ADMS', location: 'Building A • Floor 2', timezone: 7, status: 2, onlineOffset: -180 },
    { id: 2, name: 'Lobby Scanner', sn: 'JAYA9TZ0001', h_id: 4, handler: 'iClock / ADMS', location: 'Building A • Lobby', timezone: 7, status: 1, onlineOffset: -45 },
    { id: 3, name: 'Warehouse Terminal', sn: 'ATLM2TZ0002', h_id: 4, handler: 'iClock / ADMS', location: 'Building B • Warehouse', timezone: 7, status: 3, onlineOffset: -90000 },
    { id: 4, name: 'Remote Post Gateway', sn: 'RMT3TZ0003', h_id: 4, handler: 'iClock / ADMS', location: 'Atlantik • Remote Post', timezone: -2, status: 2, onlineOffset: -600 },
]

/**
 * Shifts. Offsets are seconds from LOCAL midnight, the same convention the DB
 * stores (receiver/r_shift.rs). A shift is saved with `start_time`/`end_time`
 * plus `start_enroll`/`end_enroll` margins; the backend DERIVES passday and
 * prevday from those, so the mock does the same rather than storing them.
 */
const SHIFTS = [
    { id: 1, name: 'Morning Shifts', start_time: 28800, end_time: 57600, start_enroll: 21600, end_enroll: 64800 },
    { id: 12, name: 'Half Shift', start_time: 28800, end_time: 50400, start_enroll: 21600, end_enroll: 61200 },
    { id: 13, name: 'Night Shift', start_time: 72000, end_time: 100800, start_enroll: 64800, end_enroll: 108000 },
]

/**
 * Schedules. `type` is schedule_type: 1 = flat (one shift for every day),
 * 7/14/21/28 = weekly through monthly. `dom` is schedule_dom, the day slot:
 * 1..7 in week 1, 8..14 in week 2, and so on. `parrent` groups the rows of one
 * schedule and equals the parent row's own id.
 *
 * A slot with schedule_shift_id 0 is a day off; the backend COALESCEs its name to
 * 'Libur' and the report drops that day.
 */
const SCHEDULES = [
    { id: 1, name: 'Main Office', type: 1, parrent: 1, slots: [{ dom: 1, shift: 1 }] },
    { id: 13, name: 'Weekly Office', type: 7, parrent: 13, slots: [
        { dom: 1, shift: 1 }, { dom: 2, shift: 12 }, { dom: 3, shift: 1 },
        { dom: 4, shift: 0 }, { dom: 5, shift: 12 }, { dom: 6, shift: 0 }, { dom: 7, shift: 1 },
    ] },
    { id: 27, name: 'Monthly Office', type: 28, parrent: 27, slots: [
        { dom: 1, shift: 1 }, { dom: 2, shift: 12 }, { dom: 3, shift: 1 }, { dom: 4, shift: 12 },
        { dom: 5, shift: 1 }, { dom: 6, shift: 0 }, { dom: 7, shift: 12 }, { dom: 8, shift: 1 },
        { dom: 9, shift: 0 }, { dom: 10, shift: 12 }, { dom: 11, shift: 1 }, { dom: 12, shift: 12 },
        { dom: 13, shift: 1 }, { dom: 14, shift: 0 }, { dom: 15, shift: 12 }, { dom: 16, shift: 1 },
        { dom: 17, shift: 12 }, { dom: 18, shift: 0 }, { dom: 19, shift: 1 }, { dom: 20, shift: 12 },
        { dom: 21, shift: 0 }, { dom: 22, shift: 12 }, { dom: 23, shift: 1 }, { dom: 24, shift: 0 },
        { dom: 25, shift: 12 }, { dom: 26, shift: 1 }, { dom: 27, shift: 0 }, { dom: 28, shift: 12 },
    ] },
    { id: 41, name: 'Night Watch', type: 7, parrent: 41, slots: [
        { dom: 1, shift: 13 }, { dom: 2, shift: 13 }, { dom: 3, shift: 13 },
        { dom: 4, shift: 13 }, { dom: 5, shift: 13 }, { dom: 6, shift: 13 }, { dom: 7, shift: 13 },
    ] },
]

/** Employee seed: [first, last, departement, schedule parrent id, device timezone]. */
const EMPLOYEE_SEED = [
    ['Andi', 'Wijaya', 'Riset & Pengembangan', 13, 7],
    ['Sari', 'Dewi', 'Keuangan', 13, 7],
    ['Budi', 'Santoso', 'Operasional', 1, 7],
    ['Citra', 'Lestari', 'Pemasaran', 27, 7],
    ['Dedi', 'Kurniawan', 'Riset & Pengembangan', 13, -2],
    ['Eka', 'Putri', 'Keuangan', 27, -2],
    ['Fajar', 'Nugroho', 'Operasional', 1, -2],
    ['Gita', 'Savitri', 'Pemasaran', 41, -2],
]

const STREETS = ['Melati', 'Anggrek', 'Kenanga', 'Flamboyan', 'Cendana', 'Teratai', 'Mawar', 'Daisy']
const CITIES = ['Bekasi', 'Bandung', 'Jakarta', 'Surabaya', 'Medan', 'Yogyakarta', 'Semarang', 'Denpasar']

/** Live, mutable copy of the mock database - writes land here, not in the tables. */
function buildDb() {
    return {
        employee: EMPLOYEE_SEED.map((e, i) => ({
            id: i + 1,
            first_name: e[0],
            last_name: e[1],
            address: `Jl. ${STREETS[i]} No. ${i + 3}, ${CITIES[i]}`,
            departement: e[2],
            status: i === 5 ? 1 : 0,
            schedule_id: e[3],
            timezone: e[4],
        })),
        shift: SHIFTS.map(s => ({ ...s })),
        schedule: SCHEDULES.map(s => ({ ...s })),
        device: DEVICES.map(d => ({ ...d })),
        user: DEMO_USERS.map(u => ({ ...u })),
        settings: { late_tolerance_sec: 60, overtime_tolerance_sec: 60 },
    }
}

let DB = buildDb()

/**
 * How long a demo token stays valid. Long enough that nobody browsing a demo
 * gets logged out mid-session, short enough that an old token left in a shared
 * browser eventually stops working, like the backend's session expiry.
 */
const SESSION_TTL_MS = 12 * 60 * 60 * 1000

/**
 * Upper bound on the span /report/range will materialise. The report UI asks for
 * days or weeks; anything wider is capped so a stray or hand-typed range cannot
 * build a table big enough to freeze the tab.
 */
const MAX_REPORT_DAYS = 366

// ── time helpers (ports of utility/helper.rs) ───────────────────────────

/** Deterministic 0..1 pseudo-random derived from an integer seed. */
function seeded(seed) {
    let x = (seed | 0) || 1
    x = (x ^ 61) ^ (x >>> 16)
    x = (x + (x << 3)) | 0
    x = x ^ (x >>> 4)
    x = (x * 0x27d4eb2d) | 0
    x = x ^ (x >>> 15)
    return Math.abs(x) / 0x7fffffff
}

/** Midnight of the day containing `ts`, shifted by `tz` hours - `ts_at_zero`. */
function tsAtZero(ts, tz) {
    const shifted = ts + (tz || 0) * 3600
    return shifted - (mod(shifted, 86400))
}

/** Positive modulo, so pre-1970 timestamps do not come back negative. */
function mod(n, m) {
    return ((n % m) + m) % m
}

/** ISO weekday 1..7 (Mon..Sun) - `ts_dow`. */
function tsDow(ts, tz) {
    return new Date((ts + (tz || 0) * 3600) * 1000).getUTCDay() === 0 ? 7 : new Date((ts + (tz || 0) * 3600) * 1000).getUTCDay()
}

/** ISO week number 1..52 - `ts_week`. */
function tsWeek(ts, tz) {
    const d = new Date((ts + (tz || 0) * 3600) * 1000)
    d.setUTCHours(0, 0, 0, 0)
    // Thursday of this week decides the ISO year.
    d.setUTCDate(d.getUTCDate() + 3 - ((d.getUTCDay() + 6) % 7))
    const week1 = new Date(Date.UTC(d.getUTCFullYear(), 0, 4))
    return 1 + Math.round((d.getTime() - week1.getTime()) / (7 * 86400 * 1000))
}

// ── envelope helpers (ports of utility/stor.rs) ─────────────────────────

/**
 * GenericResponse::ok - a list envelope.
 *
 * `data` is copied on the way out. A real response crosses the network, so every
 * caller gets its own array; handing back the live DB array aliased the store to
 * it, and a later push in a handler showed up in the already-rendered list. That
 * put the same row on screen twice - once because the store's array grew, once
 * because the component also pushed the row the add call returned.
 */
function listOk(data, message = 'OK') {
    return { message, data: clone(data), code: 200 }
}

/** Deep copy that keeps the payload plain JSON, exactly like a network round-trip. */
function clone(value) {
    if (Array.isArray(value)) return value.map(clone)
    if (value && typeof value === 'object') {
        const out = {}
        for (const key of Object.keys(value)) out[key] = clone(value[key])
        return out
    }
    return value
}

/** NoDataResponse::ok - a success with no payload; `data` is an empty array. */
function noDataOk(message) {
    return { message, data: [], code: 200 }
}

/** NoDataResponse::new - a failure carrying a real HTTP status. */
function fail(code, message, data) {
    return { message, data: clone(data ?? []), code }
}

// ── schedule lookup (mirrors h_report.rs get_shift_for_date) ─────────────

/**
 * All rows of a schedule, flattened from its slots into the API's shape.
 *
 * Ids follow the live DB's convention (verified against `schedules`): the FIRST
 * row of a schedule shares the parent's id, and child rows get their own ids.
 * ListSchedule.vue finds parents with `item.parrent === item.id`, so a schedule
 * whose parent row id differs from its `parrent` silently vanishes from the list.
 * /schedule/edit updates by primary key, so ids must stay unique and stable.
 */
function scheduleRows() {
    const rows = []
    // Ids are handed out sequentially but start above every parrent value, so a
    // child's id can never collide with another schedule's parent id.
    let nextId = DB.schedule.reduce((m, s) => Math.max(m, s.parrent), 0) + 1

    for (const sch of DB.schedule) {
        const isParentRow = rows.length === 0 || rows[rows.length - 1].parrent !== sch.parrent
        sch.slots.forEach((slot, index) => {
            const shift = DB.shift.find(s => s.id === slot.shift)
            const isOff = slot.shift === 0 || !shift
            const id = index === 0 && isParentRow ? sch.parrent : nextId++
            rows.push({
                id,
                name: sch.name,
                schedule_shift_id: slot.shift,
                // COALESCE(shifts.shift_name, 'Libur') in the backend.
                shift_name: isOff ? OFF_DAY : shift.name,
                start: isOff ? 0 : shift.start_time,
                end: isOff ? 0 : shift.end_time,
                type: sch.type,
                dom: slot.dom,
                parrent: sch.parrent,
            })
        })
    }
    return rows
}

/**
 * Resolve the shift a given employee works on a given day, exactly as
 * `get_shift_for_date` does: the weekday is folded into the schedule's day span,
 * the ISO week is folded into its pattern count, and the two combine into the
 * schedule_dom key. A slot that resolves to a day off returns null.
 */
function shiftForDate(dayTs, employee) {
    const sch = DB.schedule.find(s => s.parrent === Number(employee.schedule_id))
    if (!sch) return null

    const pattern = sch.type < 7 ? 1 : Math.floor(sch.type / 7)
    const domSpan = sch.type < 7 ? 1 : 7

    const dow = (tsDow(dayTs, 0) - 1) % domSpan + 1
    const week = tsWeek(dayTs, 0) % (pattern < 1 ? 1 : pattern)

    const slot = sch.slots.find(s => s.dom === dow + week * 7)
    if (!slot) return null

    const shift = DB.shift.find(s => s.id === slot.shift)
    if (slot.shift === 0 || !shift) return null

    return { ...shift, schedule_name: sch.name, schedule_type: sch.type }
}

// ── report generation (mirrors h_report.rs create_daily_report) ──────────

/**
 * Build one report row for an employee on one day.
 *
 * The frame matches the backend exactly: `enroll_date` is a UTC midnight and the
 * device timezone offset is applied ONCE - when converting the shift's local
 * wall-clock offsets into UTC epochs. Applying it twice is what stretched a shift
 * by 2x the offset and dropped days from the real report, so it is applied once.
 */
function buildReportRow(employee, dayTs) {
    const shift = shiftForDate(dayTs, employee)
    if (!shift) return null

    const tol = DB.settings
    const timezoneOffset = employee.timezone * 3600

    // Local midnight of the named calendar day, as a UTC epoch.
    const localMidnight = dayTs - timezoneOffset
    const startEnrollWindow = localMidnight + (shift.prevday === 1 ? shift.start_enroll - 86400 : shift.start_enroll)
    const endEnrollWindow = localMidnight + shift.end_enroll

    // The shift itself, in the same local frame.
    const startWorking = localMidnight + (shift.start_time >= shift.end_time ? shift.start_time - 86400 : shift.start_time)
    const endWorking = localMidnight + shift.end_time

    const dayIndex = Math.floor(dayTs / 86400)
    const seedBase = employee.id * 7919 + dayIndex * 104729

    // Attendance outcome, seeded per (employee, day) so it survives a reload.
    const roll = seeded(seedBase + 1)
    if (roll < 0.10) return null                    // absent: no punches at all

    let inPunch = null
    let outPunch = null

    if (roll < 0.22) {
        // Present but forgot to check out: only an IN punch exists.
        inPunch = startWorking + Math.floor(seeded(seedBase + 6) * 20 * 60)
        // Punches must land inside the enroll window, or the backend would not
        // have matched them to this day in the first place.
        inPunch = Math.min(Math.max(inPunch, startEnrollWindow), endEnrollWindow)
    } else {
        const lateSec = seeded(seedBase + 2) < 0.25 ? Math.floor(seeded(seedBase + 3) * 25 * 60) : 0
        const overtimeSec = seeded(seedBase + 4) < 0.18 ? Math.floor(seeded(seedBase + 5) * 45 * 60) : 0

        inPunch = Math.min(startWorking + lateSec, endWorking - 60)
        outPunch = Math.min(endWorking + overtimeSec, endEnrollWindow)
        // Keep the pair inside the window so the day is actually matched.
        inPunch = Math.max(inPunch, startEnrollWindow)
        outPunch = Math.max(outPunch, inPunch)
    }

    // Shift flags, ported from h_shift.rs: an enroll window that starts before
    // midnight or ends after shift end rolls the shift into the next day.
    const startEnroll = shift.start_time - shift.start_enroll
    const endEnroll = shift.end_time + shift.end_enroll
    const prevday = startEnroll < 0 || startEnroll > shift.end_time ? 1 : 0
    const passday = endEnroll > 86400 || endEnroll < shift.start_time ? 1 : 0

    const shiftDuration = endWorking - startWorking
    const halfShift = shiftDuration / 2

    let workingTime = 0
    let lateTime = 0
    let overtimeTime = 0

    if (inPunch !== null && inPunch > startWorking) {
        const late = inPunch - startWorking
        // Within `late_tolerance_sec` an arrival is not counted as late.
        lateTime = late <= tol.late_tolerance_sec ? 0 : Math.min(late, shiftDuration)
    }

    // Arrived past the half-shift mark without checking out: reclassify the single
    // punch as an OUT so the row still carries a duration.
    if (inPunch !== null && inPunch > startWorking + halfShift && outPunch === null) {
        outPunch = inPunch
        inPunch = null
    }

    // Working time is the ACTUAL presence between first IN and last OUT, not the
    // shift duration: a late arrival must not report a full shift, and an early
    // checkout must not hide behind one.
    if (inPunch !== null && outPunch !== null) {
        workingTime = Math.max(0, outPunch - inPunch)
    } else if (outPunch !== null) {
        workingTime = Math.max(0, outPunch - startWorking)
    }

    if (outPunch !== null && outPunch > endWorking) {
        const over = outPunch - endWorking
        overtimeTime = over <= tol.overtime_tolerance_sec ? 0 : over
    }

    // `enroll_dow` is the schedule_dom this row resolved through, so the report
    // and the schedule grid agree on which day is which.
    const pattern = shift.schedule_type < 7 ? 1 : Math.floor(shift.schedule_type / 7)
    const domSpan = shift.schedule_type < 7 ? 1 : 7
    const dow = (tsDow(dayTs, 0) - 1) % domSpan + 1
    const week = tsWeek(dayTs, 0) % (pattern < 1 ? 1 : pattern)

    return {
        id: employee.id,
        first_name: employee.first_name,
        last_name: employee.last_name,
        departement: employee.departement,
        enroll_dow: dow + week * 7,
        date: dayTs,
        tz: employee.timezone,
        enroll_device: DB.device[0].name,
        enroll_location: DB.device[0].location,
        shift_start: shift.start_time,
        shift_end: shift.end_time,
        shift_name: shift.name,
        schedule_name: shift.schedule_name,
        start: inPunch,
        end: outPunch,
        working_time: workingTime,
        late_time: lateTime,
        overtime: overtimeTime,
    }
}

/** Report rows for every employee across a UTC day range (inclusive). */
function reportRows(startTs, endTs) {
    const from = tsAtZero(startTs, 0)
    const to = tsAtZero(endTs, 0)
    const rows = []

    for (let day = from; day <= to; day += 86400) {
        for (const employee of DB.employee) {
            const row = buildReportRow(employee, day)
            if (row) rows.push(row)
        }
    }
    // The backend sorts by enroll_date before returning.
    rows.sort((a, b) => a.date - b.date)
    return rows
}

/** Recent punches for the live feed, newest first. */
function transactionLive() {
    const now = Math.floor(Date.now() / 1000)
    const rows = []

    for (let i = 0; i < 20; i++) {
        const employee = DB.employee[Math.floor(seeded(now + i * 31) * DB.employee.length)]
        const device = DB.device[Math.floor(seeded(now + i * 13) * DB.device.length)]
        rows.push({
            enroll_id: 10000 + i,
            date_time: now - Math.floor(seeded(now + i * 57) * 7200),
            first_name: employee.first_name,
            last_name: employee.last_name,
            enroll_type: 1 + Math.floor(seeded(now + i * 71) * 5),
            location: device.location,
            tz: employee.timezone,
            departement: employee.departement,
            status: seeded(now + i * 97) < 0.9 ? 1 : 0,
            employee_id: employee.id,
        })
    }
    return rows.sort((a, b) => b.date_time - a.date_time)
}

// ── permission helpers ─────────────────────────────────────────────────

/** The four action flags the permission tables store separately, all deny by default. */
function permissionRow(pageKey, canView) {
    return {
        page_key: pageKey,
        can_view: canView,
        can_add: canView,
        can_edit: canView,
        can_delete: canView,
    }
}

function userByNickname(nickname) {
    return DB.user.find(u => u.user_nickname === nickname)
}

/**
 * A request's token resolves to the demo user who owns it.
 *
 * The token format is `demo.<nickname>.<issued_at_ms>.<nonce>`: it CARRIES the
 * identity, so a page reload cannot invalidate it. An in-memory Map cannot do
 * that - `nuxt generate` emits an ES module that the browser re-evaluates on
 * every full page load, which wiped the session and bounced visitors to /login
 * the moment they refreshed or opened a second tab.
 *
 * Validity is checked structurally (prefix, a nickname that exists, an age
 * inside the demo window) rather than against a stored session, which is exactly
 * what the real backend gets from its own `sessions` table surviving a restart.
 */
function sessionUser(token) {
    if (!token || !token.startsWith('demo.')) return null
    const parts = token.split('.')
    if (parts.length !== 4) return null

    const user = userByNickname(parts[1])
    if (!user) return null

    const issuedAt = Number(parts[2])
    if (!Number.isFinite(issuedAt)) return null
    const ageMs = Date.now() - issuedAt
    if (ageMs < 0 || ageMs > SESSION_TTL_MS) return null

    return user
}

/** The nickname behind a token, or null when the session is not valid. */
function sessionNickname(token) {
    return sessionUser(token)?.user_nickname ?? null
}

/**
 * Authenticate a request and enforce a page grant. Returns null when allowed, or
 * the error envelope to return. Mirrors check_action_access: 401 with no session,
 * 403 when the page grant is missing - including View, which gates every read.
 */
function guard(token, pageKey, action) {
    const user = sessionUser(token)
    if (!user) return fail(401, 'Session token invalid', [])

    if (!pageKey) return null
    if (user.permissions[pageKey] !== 1) return fail(403, 'You do not have permission for this action', [])

    return null
}

/** Wrap a handler with a page/action gate, passing the caller's token through. */
function gated(pageKey, action, handler) {
    return (token, url, method, body) => {
        const denied = guard(token, pageKey, action)
        if (denied) return denied
        return handler(token, url, method, body)
    }
}

// ── handlers: login ─────────────────────────────────────────────────────

function login(body) {
    const errors = {}
    const nickname = body?.nickname
    const password = body?.password

    if (nickname === undefined || nickname === null) {
        errors.nickname = 'Please enter your username'
    } else if (String(nickname).length < 3) {
        errors.nickname = 'Username too sort, are you sure?'
    }

    if (password === undefined || password === null) {
        errors.password = 'Please enter your password'
    } else if (String(password).length < 8) {
        errors.password = 'Password too sort, please think again'
    }

    if (Object.keys(errors).length > 0) {
        return fail(403, 'There was an error while login', errors)
    }

    const user = userByNickname(String(nickname))
    if (!user || user.password !== String(password)) {
        return fail(403, "Username and Password combination doesn't match !", {
            nickname: 'Please re-check',
            password: 'Please re-check',
        })
    }

    // The token carries its own identity, so it survives a page reload.
    const token = `demo.${user.user_nickname}.${Date.now()}.${Math.random().toString(36).slice(2, 10)}`
    // user_id is a STRING in LoginData, not a number.
    return listOk([{ user_id: String(user.user_id), token }], 'Login success !!!')
}

/** GET /login/user - GenericResponse::single, so `data` is a one-element array. */
function loginUser(token) {
    const user = sessionUser(token)
    if (!user) return fail(400, 'Session token invalid', [])

    return listOk([{
        user_nickname: user.user_nickname,
        user_fname: user.user_fname,
        user_lname: user.user_lname,
    }], 'OK')
}

// ── handlers: employee ──────────────────────────────────────────────────

const employeeList = gated('employee', 'View', () => {
    // `timezone` is mock-only state: the employee row has no such column.
    return listOk(DB.employee.map(({ timezone, ...rest }) => rest), 'ok')
})

const employeeAdd = gated('employee', 'Add', (token, url, method, body) => {
    const errors = {}
    if (!body?.first_name) errors.first_name = 'Field required'
    if (!body?.departement) errors.departement = 'Field required'
    if (!body?.address) errors.address = 'Field required'
    if (Object.keys(errors).length > 0) return fail(403, 'There was an error while adding employee', errors)

    const dupe = DB.employee.find(e =>
        e.first_name.toLowerCase() === String(body.first_name).toLowerCase() &&
        e.last_name?.toLowerCase() === String(body.last_name || '').toLowerCase())
    if (dupe) return fail(405, 'This employee have registered!', [])

    const id = DB.employee.reduce((m, e) => Math.max(m, e.id), 0) + 1
    const row = {
        id,
        first_name: body.first_name,
        last_name: body.last_name ?? null,
        address: body.address ?? '',
        departement: body.departement,
        status: Number(body.status) || 0,
        schedule_id: Number(body.schedule_id) || 0,
        timezone: 7,
    }
    DB.employee.push(row)

    const { timezone, ...payload } = row
    return listOk([payload], 'New employee registered !!!')
})

const employeeEdit = gated('employee', 'Edit', (token, url, method, body) => {
    const row = DB.employee.find(e => e.id === Number(body?.id))
    if (!row) return fail(404, 'Employee not found', [])

    const errors = {}
    if (body?.first_name === undefined) errors.first_name = 'Field required'
    if (Object.keys(errors).length > 0) return fail(403, 'There was an error while editing employee', errors)

    row.first_name = body.first_name
    row.last_name = body.last_name ?? null
    row.address = body.address ?? ''
    row.departement = body.departement
    row.status = Number(body.status) || 0
    row.schedule_id = Number(body.schedule_id) || 0

    const { timezone, ...payload } = row
    return listOk([payload], 'Employee data updated !!!')
})

const employeeDelete = gated('employee', 'Delete', (token, url) => {
    const id = Number(routeId(url))
    const before = DB.employee.length
    DB.employee = DB.employee.filter(e => e.id !== id)
    return before === DB.employee.length
        ? fail(404, 'Employee not found', [])
        : noDataOk('Employee deleted')
})

// ── handlers: shift ─────────────────────────────────────────────────────

const shiftList = gated('shift', 'View', () => listOk(DB.shift, 'OK'))

const shiftAdd = gated('shift', 'Add', (token, url, method, body) => {
    const errors = {}
    if (!body?.name) errors.name = 'Field required'
    else if (String(body.name).length < 3) errors.name = 'Too short'
    for (const field of ['start_time', 'end_time', 'start_enroll', 'end_enroll']) {
        if (body?.[field] === undefined) errors[field] = 'Field required'
    }
    if (Object.keys(errors).length > 0) return fail(403, 'There was an error while adding shift', errors)

    if (DB.shift.some(s => s.name.toLowerCase() === String(body.name).toLowerCase())) {
        return fail(405, 'This shift name has been use!', [])
    }

    // passday/prevday are derived server-side, not accepted from the client.
    const startEnroll = Number(body.start_time) - Number(body.start_enroll)
    const endEnroll = Number(body.end_time) + Number(body.end_enroll)
    const id = DB.shift.reduce((m, s) => Math.max(m, s.id), 0) + 1

    const row = {
        id,
        name: String(body.name),
        start_time: Number(body.start_time),
        end_time: Number(body.end_time),
        start_enroll: Number(body.start_enroll),
        end_enroll: Number(body.end_enroll),
        prevday: startEnroll < 0 || startEnroll > Number(body.end_time) ? 1 : 0,
        passday: endEnroll > 86400 || endEnroll < Number(body.start_time) ? 1 : 0,
    }
    DB.shift.push(row)
    return listOk([row], 'Shift added !!!')
})

const shiftEdit = gated('shift', 'Edit', (token, url, method, body) => {
    const row = DB.shift.find(s => s.id === Number(body?.id))
    if (!row) return fail(404, 'Shift not found', [])

    const errors = {}
    if (!body?.name) errors.name = 'Field required'
    else if (String(body.name).length < 3) errors.name = 'Too short'
    if (Object.keys(errors).length > 0) return fail(403, 'There was an error while editing shift', errors)

    const startEnroll = Number(body.start_time) - Number(body.start_enroll)
    const endEnroll = Number(body.end_time) + Number(body.end_enroll)

    row.name = String(body.name)
    row.start_time = Number(body.start_time)
    row.end_time = Number(body.end_time)
    row.start_enroll = Number(body.start_enroll)
    row.end_enroll = Number(body.end_enroll)
    row.prevday = startEnroll < 0 || startEnroll > Number(body.end_time) ? 1 : 0
    row.passday = endEnroll > 86400 || endEnroll < Number(body.start_time) ? 1 : 0

    return listOk([row], 'Data shift has been update !')
})

const shiftDelete = gated('shift', 'Delete', (token, url) => {
    const id = Number(routeId(url))
    const before = DB.shift.length
    DB.shift = DB.shift.filter(s => s.id !== id)
    return before === DB.shift.length
        ? fail(404, 'Shift not found', [])
        : noDataOk('Shift deleted !!!')
})

// ── handlers: schedule ──────────────────────────────────────────────────

const scheduleList = gated('shift', 'View', () => listOk(scheduleRows(), 'OK'))

const scheduleAdd = gated('shift', 'Add', (token, url, method, body) => {
    const errors = {}

    if (!body?.name) errors.name = 'Field required'
    else if (String(body.name).length < 3) errors.name = 'Too short'

    const shifts = Array.isArray(body?.shift) ? body.shift : null
    if (!shifts || shifts.length < 1) {
        errors.shift = 'Field required'
    } else {
        // The pattern declares how many day slots the schedule needs.
        const expected = { 1: 1, 7: 7, 14: 14, 21: 21, 28: 28, 0: 1, 2: 14, 3: 21, 4: 28 }[body.pattern] ?? 0
        if (expected === 0) {
            errors.pattern = 'Invalid pattern'
        } else if (shifts.length !== expected) {
            errors.shift = `Expected ${expected} shifts for this pattern, got ${shifts.length}`
        }
    }

    if (body?.pattern === undefined || body?.pattern === null) errors.pattern = 'Field required'

    if (Object.keys(errors).length > 0) {
        return fail(403, "There's was error while adding schedules", errors)
    }

    if (DB.schedule.some(s => s.name.toLowerCase() === String(body.name).toLowerCase())) {
        return fail(405, 'This schedule name has been used!', [])
    }

    // Legacy flat patterns (0/2/3/4) are normalised to the canonical schedule_type.
    const scheduleType = { 0: 1, 1: 1, 7: 7, 14: 14, 21: 21, 28: 28, 2: 14, 3: 21, 4: 28 }[body.pattern] ?? 1
    const parrent = DB.schedule.reduce((m, s) => Math.max(m, s.parrent), 0) + 1

    DB.schedule.push({
        id: parrent,
        name: String(body.name),
        type: scheduleType,
        parrent,
        slots: shifts.map((shiftId, i) => ({ dom: i + 1, shift: Number(shiftId) || 0 })),
    })

    return noDataOk('Schedule added !!!')
})

/**
 * POST /schedule/edit - the schedule grid's drag-and-drop save. The body is
 * `{ id, shift: [new_shift_id] }` for a single row, answered with
 * NoDataResponse::ok (no payload), not a list.
 *
 * `id` is the row's PRIMARY KEY, exactly as the backend's
 * `UPDATE schedules SET schedule_shift_id=$1 WHERE schedule_id=$2` treats it -
 * not a schedule_dom. The id -> slot mapping is therefore derived from the
 * flattened rows rather than assumed.
 */
const scheduleEdit = gated('shift', 'Edit', (token, url, method, body) => {
    const shifts = Array.isArray(body?.shift) ? body.shift : null
    if (!shifts || shifts.length !== 1) {
        return fail(400, 'Format edit tidak didukung - kirim id + shift', [])
    }
    if (body?.id === undefined || Number(body.id) < 1) {
        return fail(403, 'Gagal edit jadwal', { id: 'ID jadwal wajib' })
    }

    // A shift id of 0 is a day off, which is legal and must be storable.
    const newShift = Number(shifts[0])
    const row = scheduleRows().find(r => r.id === Number(body.id))
    if (!row) return fail(404, 'Jadwal tidak ditemukan', [])

    const sch = DB.schedule.find(s => s.parrent === row.parrent)
    const slot = sch?.slots.find(s => s.dom === row.dom)
    if (!slot) return fail(404, 'Jadwal tidak ditemukan', [])
    slot.shift = newShift

    return noDataOk('Shift jadwal diperbarui')
})

const scheduleDelete = gated('shift', 'Delete', (token, url) => {
    const id = Number(routeId(url))
    const before = DB.schedule.length
    DB.schedule = DB.schedule.filter(s => s.parrent !== id)
    return before === DB.schedule.length
        ? fail(404, 'Schedule not found', [])
        : noDataOk('Schedule deleted !!!')
})

// ── handlers: device ────────────────────────────────────────────────────

/** Strip the mock-only `onlineOffset` and resolve `last_sync` to a live epoch. */
function devicePayload(d) {
    const { onlineOffset, ...rest } = d
    const now = Math.floor(Date.now() / 1000)
    return { ...rest, last_sync: now + onlineOffset }
}

const deviceList = gated('controller', 'View', () => listOk(DB.device.map(devicePayload), 'ok'))

const deviceAdd = gated('controller', 'Add', (token, url, method, body) => {
    const errors = {}
    if (!body?.name) errors.name = 'Field required'
    else if (String(body.name).length < 2) errors.name = 'Too short'
    if (!body?.sn) errors.sn = 'Field required'
    else if (String(body.sn).length < 3) errors.sn = 'Too short'
    if (!body?.location) errors.location = 'Field required'
    if (Object.keys(errors).length > 0) return fail(403, 'There was an error while adding device', errors)

    if (DB.device.some(d => d.sn === body.sn)) return fail(405, 'This devices have registered!', [])

    const id = DB.device.reduce((m, d) => Math.max(m, d.id), 0) + 1
    const row = {
        id,
        name: String(body.name),
        sn: String(body.sn),
        h_id: Number(body.handler) || 4,
        handler: 'iClock / ADMS',
        location: String(body.location),
        timezone: Number(body.timezone) || 7,
        // A freshly registered machine is offline and has never synced.
        status: 2,
        onlineOffset: 0,
    }
    DB.device.push(row)
    return listOk([devicePayload(row)], 'New devices added !!!')
})

const deviceEdit = gated('controller', 'Edit', (token, url, method, body) => {
    const row = DB.device.find(d => d.id === Number(body?.id))
    if (!row) return fail(404, 'Device not found', [])

    const errors = {}
    if (!body?.name) errors.name = 'Field required'
    else if (String(body.name).length < 2) errors.name = 'Too short'
    if (!body?.sn) errors.sn = 'Field required'
    if (Object.keys(errors).length > 0) return fail(403, 'There was an error while editing device', errors)

    row.name = String(body.name)
    row.sn = String(body.sn)
    row.h_id = Number(body.handler) || 4
    row.location = String(body.location ?? '')
    row.timezone = Number(body.timezone) || 7
    // device_state is server-owned: the heartbeat and the sweeper own it, and a
    // disabled machine (state 3) must never be brought back online by a write.
    return listOk([devicePayload(row)], 'Device data updated !!!')
})

const deviceDelete = gated('controller', 'Delete', (token, url) => {
    const id = Number(routeId(url))
    const before = DB.device.length
    DB.device = DB.device.filter(d => d.id !== id)
    return before === DB.device.length
        ? fail(404, 'Device not found', [])
        : noDataOk('Device deleted')
})

// ── handlers: report ────────────────────────────────────────────────────

const reportToday = gated('report', 'View', () => {
    const today = tsAtZero(Math.floor(Date.now() / 1000), 0)
    return listOk(reportRows(today, today), 'OK')
})

const reportRange = gated('report', 'View', (token, url) => {
    const [, start, end] = /^\/report\/range\/(-?\d+)\/(-?\d+)$/.exec(routeOf(url)) || []
    const from = tsAtZero(Number(start), 0)
    const to = tsAtZero(Number(end), 0)

    // A range with no upper bound (or an absurd one) would materialise hundreds
    // of thousands of rows and lock up the browser tab. The UI only ever asks for
    // days or weeks, so cap it and answer exactly what the window can hold.
    const cappedTo = Math.min(to, from + (MAX_REPORT_DAYS - 1) * 86400)
    return listOk(reportRows(from, cappedTo), 'OK')
})

// ── handlers: settings ──────────────────────────────────────────────────

const settingsList = gated('settings', 'View', () => listOk([
    { setting_key: 'late_tolerance_sec', setting_value: DB.settings.late_tolerance_sec },
    { setting_key: 'overtime_tolerance_sec', setting_value: DB.settings.overtime_tolerance_sec },
], 'OK'))

const settingsEdit = gated('settings', 'Edit', (token, url, method, body) => {
    const errors = {}

    // Validate both fields before writing either, so a rejected value never
    // half-applies - the same contract as h_settings.rs.
    for (const key of ['late_tolerance_sec', 'overtime_tolerance_sec']) {
        const value = body?.[key]
        if (value === undefined || value === null) continue
        if (!Number.isFinite(Number(value)) || Number(value) < 0 || Number(value) > MAX_TOL) {
            errors[key] = `Value must be between 0 and ${MAX_TOL}`
        }
    }
    if (Object.keys(errors).length > 0) return fail(400, 'There was an error while saving', errors)

    for (const key of ['late_tolerance_sec', 'overtime_tolerance_sec']) {
        if (body?.[key] !== undefined && body?.[key] !== null) DB.settings[key] = Number(body[key])
    }
    return noDataOk('Settings saved !!!')
})

// ── handlers: user ──────────────────────────────────────────────────────

const userList = gated('users', 'View', () => {
    // UserWithPermissions.permissions is a MAP keyed by page_key, not an array.
    return listOk(DB.user.map(u => ({
        user_id: u.user_id,
        user_nickname: u.user_nickname,
        user_fname: u.user_fname,
        user_lname: u.user_lname,
        permissions: PAGE_KEYS.reduce((acc, page_key) => {
            acc[page_key] = permissionRow(page_key, u.permissions[page_key] === 1)
            return acc
        }, {}),
    })), 'ok')
})

const userAdd = gated('users', 'Add', (token, url, method, body) => {
    const errors = {}
    if (!body?.nickname) errors.nickname = 'Field required'
    else if (String(body.nickname).length < 3) errors.nickname = 'Too short'
    if (!body?.password) errors.password = 'Field required'
    else if (String(body.password).length < 8) errors.password = 'Too short'
    if (!body?.fname) errors.fname = 'Field required'
    else if (String(body.fname).length < 2) errors.fname = 'Too short'
    if (Object.keys(errors).length > 0) return fail(403, 'There was an error while adding user', errors)

    if (userByNickname(String(body.nickname))) return fail(405, 'This username has been used!', [])

    const id = DB.user.reduce((m, u) => Math.max(m, u.user_id), 0) + 1
    DB.user.push({
        user_id: id,
        user_nickname: String(body.nickname),
        user_fname: String(body.fname),
        user_lname: String(body.lname ?? ''),
        password: String(body.password),
        // A new account starts with no pages: view is never granted implicitly.
        permissions: PAGE_KEYS.reduce((acc, page_key) => { acc[page_key] = 0; return acc }, {}),
    })
    // GenericResponse<String>::ok(vec![user_nickname]) - the payload is the nickname.
    return listOk([String(body.nickname)], 'User added')
})

const userEdit = gated('users', 'Edit', (token, url, method, body) => {
    const target = userByNickname(String(body?.user_nickname))
    if (!target) return fail(400, 'user_nickname required', [])

    const errors = {}
    if (body?.nickname !== undefined) {
        if (String(body.nickname).length < 3) errors.nickname = 'Too short'
    }
    if (body?.password !== undefined && String(body.password).length < 8) errors.password = 'Too short'
    if (body?.fname !== undefined && String(body.fname).length < 2) errors.fname = 'Too short'

    if (Object.keys(errors).length > 0) return fail(400, 'There was an error while editing user', errors)

    const updates = []
    if (body?.nickname !== undefined) {
        target.user_nickname = String(body.nickname)
        updates.push('nickname')
    }
    if (body?.password !== undefined) {
        target.password = String(body.password)
        updates.push('password')
    }
    if (body?.fname !== undefined) {
        target.user_fname = String(body.fname)
        updates.push('fname')
    }
    if (body?.lname !== undefined) {
        target.user_lname = String(body.lname)
        updates.push('lname')
    }

    if (updates.length === 0) return fail(400, 'No fields to update', [])
    // NoDataResponse::new("OK", 200) - no payload on an edit.
    return noDataOk('OK')
})

const userDelete = gated('users', 'Delete', (token, url, method, body) => {
    const currentNickname = sessionNickname(token)
    if (!body?.user_nickname) return fail(400, 'user_nickname required', [])

    // Deleting your own row would revoke your own permissions mid-request and
    // leave nobody able to undo it.
    if (currentNickname === body.user_nickname) {
        return fail(400, 'Cannot delete yourself', [])
    }

    const target = userByNickname(String(body.user_nickname))
    if (!target) return fail(404, 'User not found', [])

    DB.user = DB.user.filter(u => u.user_id !== target.user_id)
    return noDataOk('OK')
})

/**
 * POST /user/permissions - update another account's flags.
 *
 * Self-lockout is refused: with no admin level and no superuser, dropping your own
 * `users` access is a one-way door. can_view AND can_edit must both survive, since
 * this very endpoint is gated on can_edit - dropping it would succeed once and
 * then lock you out of every later permission change.
 */
const userPermissionsUpdate = gated('users', 'Edit', (token, url, method, body) => {
    const currentNickname = sessionNickname(token)
    if (!body?.user_nickname) return fail(400, 'user_nickname required', [])

    const target = userByNickname(String(body.user_nickname))
    if (!target) return fail(404, 'User not found', [])

    const incoming = Array.isArray(body.permissions) ? body.permissions : []

    if (currentNickname === target.user_nickname) {
        let keepsView = false
        let keepsEdit = false
        for (const p of incoming) {
            if (p?.page_key === 'users') {
                keepsView = p.can_view === true
                keepsEdit = p.can_edit === true
            }
        }
        if (!keepsView) return fail(400, 'You cannot remove your own user management access', [])
        if (!keepsEdit) return fail(400, 'You cannot remove your own edit access to user management', [])
    }

    for (const p of incoming) {
        const pageKey = p?.page_key
        if (!PAGE_KEYS.includes(pageKey)) continue
        // A page you cannot read is not a page you can write to, so the four
        // flags are kept consistent with the view grant, as users.vue assumes.
        const canView = p.can_view === true
        target.permissions[pageKey] = canView ? 1 : 0
    }
    return noDataOk('OK')
})

/**
 * GET /user/permissions - the CALLER's own flags, as a flat array of
 * UserPermission rows. Deliberately not gated on the 'users' page: every signed-in
 * user needs their own flags to render the sidebar and guard their routes.
 */
function userPermissionsList(token) {
    const nickname = sessionNickname(token)
    if (!nickname) return fail(401, 'Session token invalid', [])

    const user = userByNickname(nickname)
    if (!user) return fail(401, 'Session token invalid', [])

    // ORDER BY page_key ASC, like the backend query.
    const rows = [...PAGE_KEYS].sort().map(page_key => permissionRow(page_key, user.permissions[page_key] === 1))
    return listOk(rows, 'OK')
}

// ── dispatcher ──────────────────────────────────────────────────────────

/**
 * Strip the origin (when the app calls an absolute URL), the `/api` prefix, and
 * any trailing slash or query. Accepts `/login`, `/api/login`, and
 * `http://host:port/login` alike, since `apiBase` may be empty, a proxy path, or
 * a full origin depending on the deployment.
 */
function routeOf(url) {
    let path = String(url)
    try {
        // URL throws on relative paths, which is exactly the case we skip.
        const parsed = new URL(path)
        path = parsed.pathname + parsed.search
    } catch {
        /* relative path: use as-is */
    }
    path = path.split('?')[0]
    const withoutApi = path.startsWith('/api/') ? path.slice(4) : path
    return withoutApi.replace(/\/+$/, '')
}

/** Trailing path segment, e.g. `/shift/delete/7` -> `7`. */
function routeId(url) {
    return routeOf(url).split('/').pop()
}

/** Read a request body that may arrive as a JSON string or a plain object. */
function bodyOf(options) {
    const body = options?.body
    if (body === undefined || body === null) return {}
    if (typeof body === 'string') {
        try { return JSON.parse(body) } catch { return {} }
    }
    if (typeof body === 'object') return body
    return {}
}

/** Pull the bearer token out of the headers the app's `confHeaders()` builds. */
function tokenOf(options) {
    const auth = options?.headers?.Authorization || options?.headers?.authorization
    if (!auth) return null
    return String(auth).replace(/^Bearer\s+/i, '')
}

function dispatch(url, options) {
    const route = routeOf(url)
    const method = String(options?.method || 'GET').toUpperCase()
    const token = tokenOf(options)
    const body = bodyOf(options)
    const call = (handler) => handler(token, url, method, body)

    // ── login / session ──
    if (route === '/login' && method === 'POST') return login(body)
    if (route === '/login/user' && method === 'GET') return loginUser(token)
    if (route === '/login/logout' && method === 'GET') {
        // Nothing to destroy server-side: the token is self-describing and the
        // cookie is deleted by the client, so a logged-out browser can no longer
        // present it.
        return noDataOk('OK')
    }
    if (route === '/user/permissions' && method === 'GET') return userPermissionsList(token)
    if (route === '/user/permissions' && method === 'POST') return call(userPermissionsUpdate)

    // ── employee ──
    if (route === '/employee/list' && method === 'GET') return call(employeeList)
    if (route === '/employee/add' && method === 'POST') return call(employeeAdd)
    if (route === '/employee/edit' && method === 'POST') return call(employeeEdit)
    if (/^\/employee\/delete\/\d+$/.test(route)) return call(employeeDelete)

    // ── shift ──
    if (route === '/shift/list' && method === 'GET') return call(shiftList)
    if (route === '/shift/add' && method === 'POST') return call(shiftAdd)
    if (route === '/shift/edit' && method === 'POST') return call(shiftEdit)
    if (/^\/shift\/delete\/\d+$/.test(route)) return call(shiftDelete)

    // ── schedule ──
    if (route === '/schedule/list' && method === 'GET') return call(scheduleList)
    if (route === '/schedule/add' && method === 'POST') return call(scheduleAdd)
    if (route === '/schedule/edit' && method === 'POST') return call(scheduleEdit)
    if (/^\/schedule\/delete\/\d+$/.test(route)) return call(scheduleDelete)

    // ── device ──
    if (route === '/device/list' && method === 'GET') return call(deviceList)
    if (route === '/device/add' && method === 'POST') return call(deviceAdd)
    if (route === '/device/edit' && method === 'POST') return call(deviceEdit)
    if (/^\/device\/delete\/\d+$/.test(route)) return call(deviceDelete)

    // ── report ──
    if (route === '/report/today' && method === 'GET') return call(reportToday)
    if (/^\/report\/range\/(-?\d+)\/(-?\d+)$/.test(route)) return call(reportRange)

    // ── settings ──
    if (route === '/settings/list' && method === 'GET') return call(settingsList)
    if (route === '/settings/edit' && method === 'POST') return call(settingsEdit)

    // ── user ──
    if (route === '/user/list' && method === 'GET') return call(userList)
    if (route === '/user/add' && method === 'POST') return call(userAdd)
    if (route === '/user/edit' && method === 'POST') return call(userEdit)
    if (route === '/user/delete' && method === 'POST') return call(userDelete)

    // ── live feed ──
    // Gated on 'dashboard', not 'report': the dashboard renders this feed.
    if (route === '/transaction/live' && method === 'GET') {
        const denied = guard(token, 'dashboard', 'View')
        if (denied) return denied
        const rows = transactionLive()
        // The newest enroll_id travels in `message` - LiveTable.vue parses it.
        return listOk(rows, String(rows[0]?.enroll_id ?? 0))
    }

    return fail(404, 'Not found', [])
}

/**
 * Answer a mock request with the response envelope the real backend would send.
 * Returns the same `{ message, data, code }` shape `apiFetch` hands to callers, so
 * no page can tell the difference.
 */
export function mockRequest(url, options) {
    try {
        return dispatch(url, options)
    } catch (err) {
        // The backend never reports an error with HTTP 200, and neither does this.
        console.error('[demo mock]', url, err)
        return fail(500, 'Internal server error', [])
    }
}

/** Reset all mock state (for a "reset demo" affordance, and between test runs). */
export function mockReset() {
    DB = buildDb()
}