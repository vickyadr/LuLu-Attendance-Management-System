<script setup>
import { usePayrollStore } from '~/store/payroll';
import { useAuthStore } from '~/store/auth';
import { useNotifyStore } from '~/store/notify';
import { useChecker, useFormater } from '#imports';
import { SMData } from '~/components/SimpleModal.vue';
import { faInbox, faRotate, faXmark } from '@fortawesome/free-solid-svg-icons';

const payroll = usePayrollStore();
const auth = useAuthStore();
const check = useChecker();
const format = useFormater();
const config = useRuntimeConfig();

useHead({ title: "LuLu - Payroll" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

const { t } = useI18n();
const tab = ref("generate"); // "generate", "list", "salary"

// Generate form
const periodStart = ref(null);
const periodEnd = ref(null);
const genScope = ref("all"); // "all" | "employee" | "department"
const genEmployeeId = ref("");
const genEmpQuery = ref("");
const genEmpOpen = ref(false);
const genEmpIdx = ref(-1);
const genEmpFilter = computed(() => {
    const q = genEmpQuery.value.trim().toLowerCase();
    const list = employees.value || [];
    if (!q) return list.slice(0, 8);
    return list.filter((e) => salaryEmpLabel(e).toLowerCase().includes(q)).slice(0, 8);
});
const genDepartment = ref("");
const genDeptQuery = ref("");
const genDeptOpen = ref(false);
const genDeptIdx = ref(-1);
const genDeptFilter = computed(() => {
    const q = genDeptQuery.value.trim().toLowerCase();
    if (!q) return salaryDepts.value.slice(0, 8);
    return salaryDepts.value.filter((d) => d.toLowerCase().includes(q)).slice(0, 8);
});
const dailyConditions = ref([]);
const showConditionInput = ref(false);
const newCondition = ref({ employee_id: "", work_date: null, condition: "", reason: "" });

// Typeahead for the daily-condition employee picker. The select listed the
// whole roster, which stops being usable once a company has more than a few
// dozen people.
const empQuery = ref("");
const empOpen = ref(false);
const empMatches = computed(() => {
    const q = empQuery.value.trim().toLowerCase();
    const people = employees.value;
    if (!q) return people.slice(0, 8);
    return people.filter((e) => {
        const name = `${e.first_name || e.name || ""} ${e.last_name || ""}`.toLowerCase();
        const dept = String(e.department || "").toLowerCase();
        return name.includes(q) || dept.includes(q) || String(e.id) === q || String(e.nickname || "").toLowerCase().includes(q);
    }).slice(0, 8);
});
function pickConditionEmployee(e) {
    newCondition.value.employee_id = e.id;
    empQuery.value = [e.first_name || e.name, e.last_name].filter(Boolean).join(" ");
    empOpen.value = false;
}
function onEmpInput() {
    // Typing invalidates the previous pick, so the add button cannot submit a
    // name that no longer matches the chosen id.
    newCondition.value.employee_id = "";
    empOpen.value = true;
}
const generating = ref(false);
const generateError = ref(null);
const generateResult = ref(null);

// List view
const payslips = ref([]);
const listLoading = ref(false);

// Detail modal + CSV export (client-side, no dep)
const detailSlip = ref(null);
const detailLoading = ref(false);
const detailError = ref(null);
const detailModal = new SMData();

// Payslip detail modal: BE returns a flat header (flattened document),
// enriched list rows omit lines; normalize all three shapes here.
function normalizeDetailSlip(doc) {
    if (!doc) return doc;
    if (doc.payslip) return { ...doc.payslip, lines: doc.lines || [] };
    if (doc.id && !doc.lines) return { ...doc, lines: [] };
    return doc;
}

// Thousands separators for money: 3590000 -> 3.590.000
function fmtMoney(v) {
    if (v === null || v === undefined || v === '') return '-'
    const n = Number(String(v).replace(/[^0-9.-]/g, ''))
    if (!Number.isFinite(n)) return String(v)
    return n.toLocaleString('id-ID', { maximumFractionDigits: 0 })
}

async function openDetail(id) {
    detailSlip.value = null;
    detailError.value = null;
    detailLoading.value = true;
    detailModal.setText(t("payroll.detail"), "");
    detailModal.showCancel();
    try {
        detailSlip.value = normalizeDetailSlip(await payroll.getDetail(id));
    } catch (e) {
        detailError.value = e.message;
    } finally {
        detailLoading.value = false;
    }
}

function closeDetail() {
    detailSlip.value = null;
    detailError.value = null;
    detailModal.clear();
}

function exportCsv() {
    if (!detailSlip.value) return;
    const s = detailSlip.value;
    const cell = csvCell;
    const rows = [
        ["employee_id", "period_start", "period_end", "regular_days_pay", "ot_pay", "holiday_premium_pay", "paid_leave_pay", "gross_salary", "total_deductions", "net_salary", "status"],
        [s.employee_id, s.period_start, s.period_end, s.regular_days_pay, s.ot_pay, s.holiday_premium_pay, s.paid_leave_pay, s.gross_salary, s.total_deductions, s.net_salary, s.status],
        [],
        ["work_date", "days_credited", "ot_hours", "daily_pay", "ot_pay", "holiday_premium"],
        ...(s.lines || []).map(l => [l.work_date, l.days_credited, l.ot_hours, l.daily_pay, l.ot_pay, l.holiday_premium]),
    ];
    const csv = rows.map(r => r.map(cell).join(",")).join("\n");
    const blob = new Blob([csv], { type: "text/csv" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `payslip-${s.employee_id}-${s.period_start}.csv`;
    a.click();
    URL.revokeObjectURL(a.href);
}

// Salary tab: assign salary to unblock generate gate (monthly/daily + manual rates)
const employees = ref([]);
const salaryEmpId = ref("");
const salaryEmpQuery = ref("");
const salaryEmpOpen = ref(false);
const salaryEmpIdx = ref(-1);
const salaryEmpFilter = computed(() => {
    const q = salaryEmpQuery.value.trim().toLowerCase();
    const list = employees.value || [];
    if (!q) return list.slice(0, 8);
    return list.filter((e) => salaryEmpLabel(e).toLowerCase().includes(q)).slice(0, 8);
});
// Salary tab filter (employee autocomplete + All/Newest/Old)
const salaryFilterQuery = ref("");
const salaryFilterOpen = ref(false);
const salaryFilterIdx = ref(-1);
const salaryFilterEmpId = ref(null);
const salaryMode = ref("all");
const salaryFilterList = computed(() => {
    const q = salaryFilterQuery.value.trim().toLowerCase();
    const list = employees.value || [];
    if (!q) return list.slice(0, 8);
    return list.filter((e) => salaryEmpLabel(e).toLowerCase().includes(q)).slice(0, 8);
});
const filteredSalaryTable = computed(() => {
    let rows = salaryTable.value || [];
    if (salaryFilterEmpId.value) rows = rows.filter((r) => String(r.employee.id) === String(salaryFilterEmpId.value));
    if (salaryMode.value === "all") return rows;
    // newest/old: one row per employee (by effective_date)
    const byEmp = new Map();
    for (const r of rows) {
        if (!r.salary) continue;
        const prev = byEmp.get(String(r.employee.id));
        const d = r.salary.effective_date;
        if (!prev) { byEmp.set(String(r.employee.id), r); continue; }
        const pd = prev.salary.effective_date;
        if (salaryMode.value === "newest" ? d > pd : d < pd) byEmp.set(String(r.employee.id), r);
    }
    return [...byEmp.values()];
});
// import state
const importFile = ref(null);
const importLoading = ref(false);
const importResult = ref(null);
const importConfirmModal = new SMData();
let pendingImportCsv = null;
function downloadSalaryTemplate() {
    const csv = "EM,ID,TYPE,EFFECTIVE_DATE,EMPLOYEE_NAME,MONTLY_RATE,DAILY_RATE,HOURLY_RATE\nRL,RULE_NAME,RULE_TYPE,RULE_FORMULA,IS_DAILY,,,\nEM,1,MONTHLY,01/11/2026,Employee Name,5000000,165000,24000\nRL,Gaji Pokok,ADD,monthly_rate,0,,,\n";
    const blob = new Blob([csv], { type: "text/csv" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "salary-import-template.csv";
    a.click();
    URL.revokeObjectURL(a.href);
}
function setImportResult(res) {
    // Backend rejects the whole file on rule/parse errors; show every bad row.
    if (res?.data?.errors?.length) {
        // `error` is the backend's i18n key; fall back to its English message.
        importResult.value = { error: res.error ? t(res.error) : res.message, errors: res.data.errors };
        return;
    }
    // A 413 from the body-size limit has no `data`, and a bare `data: []` from
    // other failures is truthy: branch on the code before trusting the payload,
    // otherwise the summary renders with undefined counts.
    if (res?.code >= 400 || !res?.data || Array.isArray(res.data)) {
        importResult.value = { error: res?.error ? t(res.error) : (res?.message || "Import failed") };
        return;
    }
    importResult.value = res.data;
}
async function handleImportCsv() {
    if (!importFile.value) return;
    const text = await importFile.value.text();
    importLoading.value = true;
    importResult.value = null;
    try {
        const res = await apiFetch(`${config.public.apiBase}/payroll/employee-salaries/import`, {
            method: "POST",
            headers: auth.confHeaders(),
            body: { csv: text, confirm: false },
        });
        if (res?.data?.needs_confirm) {
            pendingImportCsv = text;
            importConfirmModal.setText(t("payroll.importConfirmTitle"), `${t("payroll.importConfirmReplace")} (${(res.data.duplicates || []).map(d => `${d.id} ${d.name}`).join(", ")})`);
            importConfirmModal.showOKCancel();
            return;
        }
        setImportResult(res);
        if (!importResult.value?.errors) await loadSalaryTable();
    } catch (e) {
        importResult.value = { error: e.message };
    } finally {
        importLoading.value = false;
    }
}
async function confirmImportReplace() {
    if (!pendingImportCsv) return;
    importConfirmModal.clear();
    importLoading.value = true;
    try {
        const res = await apiFetch(`${config.public.apiBase}/payroll/employee-salaries/import`, {
            method: "POST",
            headers: auth.confHeaders(),
            body: { csv: pendingImportCsv, confirm: true },
        });
        setImportResult(res);
        pendingImportCsv = null;
        await loadSalaryTable();
    } catch (e) {
        importResult.value = { error: e.message };
    } finally {
        importLoading.value = false;
    }
}
function cancelImportReplace() {
    pendingImportCsv = null;
    importConfirmModal.clear();
}
const salaryType = ref("monthly");
const salaryMonthly = ref("");
const salaryDaily = ref("");
const salaryHourly = ref("");
const salaryBasic = ref("");
const salaryDept = ref("");
const salaryEffective = ref("");
const salaryCurrent = ref(null);
const salarySaving = ref(false);
const salaryError = ref(null);
const salaryTable = ref([]);
const salaryTableLoading = ref(false);
const salaryModal = new SMData();
const rulesHelpModal = new SMData();
const salaryRules = ref([]);
const salaryCopyFrom = ref("");
const salaryCopyLoading = ref(false);

// New rule uses agreed 3-col shape: name + function kind combobox + parameter + daily flag.
function salaryNewRule() {
    return { name: "", function: "Add", add: true, deduct: false, parameter: "", daily: false };
}

// Sync add/deduct flags from the function kind combobox (Add or Deduct).
function onRuleFunction(rule) {
    const isDeduct = String(rule.function || "").toLowerCase() === "deduct";
    rule.add = !isDeduct;
    rule.deduct = isDeduct;
}

function normalizeLoadedRule(r) {
    // Map stored add/deduct flags onto the function combobox value for display.
    // Legacy rows carry a formula in function, so move it to parameter.
    const kind = r.deduct ? "Deduct" : "Add";
    if (r.function != null && (r.function === "Add" || r.function === "Deduct")) {
        return {
            name: r.name || "",
            function: r.function,
            add: r.function !== "Deduct",
            deduct: r.function === "Deduct",
            parameter: r.parameter || "",
            daily: !!r.daily,
        };
    }
    if (r.function != null) {
        return {
            name: r.name || "",
            function: kind,
            add: r.add !== false && !r.deduct,
            deduct: !!r.deduct,
            parameter: r.parameter || String(r.function || ""),
            daily: !!r.daily,
        };
    }
    const basis = ["absent", "present", "overtime"].includes(r.basis) ? r.basis : "fixed";
    const amount = r.amount != null ? String(r.amount) : "";
    const mult = r.multiplier != null && Number(r.multiplier) !== 1 ? ` * ${r.multiplier}` : "";
    return {
        name: r.name || "",
        function: r.kind === "deduct" ? "Deduct" : "Add",
        add: r.kind !== "deduct",
        deduct: r.kind === "deduct",
        parameter: basis === "fixed" ? `${amount}${mult}`.trim() : `${basis} * ${amount || 0}${mult}`.trim(),
        daily: basis !== "fixed",
    };
}

function addSalaryRule() {
    salaryRules.value.push(salaryNewRule());
}

function removeSalaryRule(idx) {
    salaryRules.value.splice(idx, 1);
}

// Reorder rules: sequence matters, later rules see salary mutated by earlier ones.
function moveSalaryRule(idx, dir) {
    const to = idx + dir;
    if (to < 0 || to >= salaryRules.value.length) return;
    const arr = salaryRules.value;
    const [item] = arr.splice(idx, 1);
    arr.splice(to, 0, item);
}

async function copySalaryRules() {
    salaryError.value = null;
    if (!salaryCopyFrom.value) return;
    salaryCopyLoading.value = true;
    try {
        const src = await payroll.getSalary(salaryCopyFrom.value);
        const srcRules = src && src.rules_json ? src.rules_json : [];
        salaryRules.value = (Array.isArray(srcRules) ? srcRules : []).map(normalizeLoadedRule);
    } catch (e) {
        salaryError.value = e.message;
    } finally {
        salaryCopyLoading.value = false;
    }
}

// Shared rules payload builder: kind label in function, formula in parameter.
function buildRulesPayload(rows) {
    return (rows || salaryRules.value || [])
        .filter((r) => String(r.name || "").trim() !== "" || String(r.parameter || "").trim() !== "")
        .map((r) => {
            const kind = String(r.function || "Add").trim() || "Add";
            const param = String(r.parameter || "").trim();
            const isDeduct = kind.toLowerCase() === "deduct";
            return {
                name: String(r.name || "").trim(),
                function: isDeduct ? "Deduct" : "Add",
                add: !isDeduct,
                deduct: isDeduct,
                parameter: param === "" ? null : param,
                daily: !!r.daily,
            };
        });
}

// Dept suggest: distinct departments from loaded employees (employee page pattern)
const salaryDepts = computed(() => {
    const s = new Set((employees.value || []).map((x) => x.department).filter(Boolean));
    return [...s].sort();
});

// Employee suggest: name + dept match, capped so dropdown stays short
function salaryEmpLabel(e) {
    const nm = [e.first_name || e.name, e.last_name].filter(Boolean).join(" ");
    return e.department ? `${nm} - ${e.department}` : nm;
}
// Employee suggest: custom dropdown list (datalist is unreliable on mobile)

function salaryEmpName(id) {
    const e = (employees.value || []).find((x) => String(x.id) === String(id));
    return e ? salaryEmpLabel(e) : "";
}

function onSalaryEmpInput() {
    matchSalaryEmp();
    salaryEmpOpen.value = true;
    salaryEmpIdx.value = -1;
}

function onSalaryEmpFocus() {
    if (!employees.value.length) loadEmployees();
    salaryEmpOpen.value = true;
    salaryEmpIdx.value = -1;
}

function onSalaryEmpKey(e) {
    const n = salaryEmpFilter.value.length;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        if (!salaryEmpOpen.value) { salaryEmpOpen.value = true; return; }
        if (!n) return;
        salaryEmpIdx.value = e.key === "ArrowDown"
            ? Math.min(salaryEmpIdx.value + 1, n - 1)
            : Math.max(salaryEmpIdx.value - 1, 0);
        nextTick(() => document.querySelector(".salary-emp-item.is-active")?.scrollIntoView({ block: "nearest" }));
    } else if (e.key === "Enter") {
        const p = salaryEmpIdx.value >= 0 ? salaryEmpFilter.value[salaryEmpIdx.value] : undefined;
        if (!p) return;
        e.preventDefault();
        pickSalaryEmp(p);
    } else if (e.key === "Escape") {
        salaryEmpOpen.value = false;
        salaryEmpIdx.value = -1;
    }
}

function matchSalaryEmp() {
    const q = salaryEmpQuery.value.trim().toLowerCase();
    const hit = q
        ? (employees.value || []).find((e) => salaryEmpLabel(e).toLowerCase() === q
            || salaryEmpLabel(e).toLowerCase().includes(q))
        : null;
    salaryEmpId.value = hit ? hit.id : "";
    if (hit) { if (!salaryCurrent.value || String(salaryCurrent.value.employee_id) !== String(hit.id)) lookupSalary(); return; }
    salaryCurrent.value = null;
}

function onSalaryFilterInput() { matchSalaryFilter(); salaryFilterOpen.value = true; salaryFilterIdx.value = -1; }
function onSalaryFilterFocus() { if (!employees.value.length) loadEmployees(); salaryFilterOpen.value = true; salaryFilterIdx.value = -1; }
function onSalaryFilterKey(e) {
    const n = salaryFilterList.value.length;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        if (!salaryFilterOpen.value) { salaryFilterOpen.value = true; return; }
        if (!n) return;
        salaryFilterIdx.value = e.key === "ArrowDown"
            ? Math.min(salaryFilterIdx.value + 1, n - 1)
            : Math.max(salaryFilterIdx.value - 1, 0);
    } else if (e.key === "Enter") {
        const p = salaryFilterIdx.value >= 0 ? salaryFilterList.value[salaryFilterIdx.value] : undefined;
        if (!p) return;
        e.preventDefault();
        salaryFilterQuery.value = salaryEmpLabel(p);
        salaryFilterEmpId.value = p.id;
        salaryFilterOpen.value = false;
        salaryFilterIdx.value = -1;
    }
}
function matchSalaryFilter() {
    const q = salaryFilterQuery.value.trim().toLowerCase();
    const hit = q ? (employees.value || []).find((e) => salaryEmpLabel(e).toLowerCase() === q || salaryEmpLabel(e).toLowerCase().includes(q)) : null;
    salaryFilterEmpId.value = hit ? hit.id : null;
}

// Generate scope suggest: same custom dropdown pattern as salary form.
function onGenEmpInput() { matchGenEmp(); genEmpOpen.value = true; genEmpIdx.value = -1; }
function onGenEmpFocus() { if (!employees.value.length) loadEmployees(); genEmpOpen.value = true; genEmpIdx.value = -1; }
function onGenEmpKey(e) {
    const n = genEmpFilter.value.length;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        if (!genEmpOpen.value) { genEmpOpen.value = true; return; }
        if (!n) return;
        genEmpIdx.value = e.key === "ArrowDown" ? Math.min(genEmpIdx.value + 1, n - 1) : Math.max(genEmpIdx.value - 1, 0);
        nextTick(() => document.querySelector(".gen-emp-item.is-active")?.scrollIntoView({ block: "nearest" }));
    } else if (e.key === "Enter") {
        const p = genEmpIdx.value >= 0 ? genEmpFilter.value[genEmpIdx.value] : undefined;
        if (!p) return;
        e.preventDefault();
        pickGenEmp(p);
    } else if (e.key === "Escape") { genEmpOpen.value = false; genEmpIdx.value = -1; }
}
function matchGenEmp() {
    const q = genEmpQuery.value.trim().toLowerCase();
    const hit = q ? (employees.value || []).find((e) => salaryEmpLabel(e).toLowerCase() === q || salaryEmpLabel(e).toLowerCase().includes(q)) : null;
    genEmployeeId.value = hit ? hit.id : "";
}
function pickGenEmp(e) { genEmployeeId.value = e.id; genEmpQuery.value = salaryEmpLabel(e); genEmpOpen.value = false; genEmpIdx.value = -1; }
function onGenDeptInput() { matchGenDept(); genDeptOpen.value = true; genDeptIdx.value = -1; }
function onGenDeptFocus() { genDeptOpen.value = true; genDeptIdx.value = -1; }
function onGenDeptKey(e) {
    const n = genDeptFilter.value.length;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        if (!genDeptOpen.value) { genDeptOpen.value = true; return; }
        if (!n) return;
        genDeptIdx.value = e.key === "ArrowDown" ? Math.min(genDeptIdx.value + 1, n - 1) : Math.max(genDeptIdx.value - 1, 0);
        nextTick(() => document.querySelector(".gen-dept-item.is-active")?.scrollIntoView({ block: "nearest" }));
    } else if (e.key === "Enter") {
        const p = genDeptIdx.value >= 0 ? genDeptFilter.value[genDeptIdx.value] : undefined;
        if (!p) return;
        e.preventDefault();
        pickGenDept(p);
    } else if (e.key === "Escape") { genDeptOpen.value = false; genDeptIdx.value = -1; }
}
function matchGenDept() {
    const q = genDeptQuery.value.trim().toLowerCase();
    const hit = q ? salaryDepts.value.find((d) => d.toLowerCase() === q || d.toLowerCase().includes(q)) : null;
    genDepartment.value = hit || "";
}
function pickGenDept(d) { genDepartment.value = d; genDeptQuery.value = d; genDeptOpen.value = false; genDeptIdx.value = -1; }

function pickSalaryEmp(e) {
    salaryEmpId.value = e.id;
    salaryEmpQuery.value = salaryEmpLabel(e);
    salaryEmpOpen.value = false;
    salaryEmpIdx.value = -1;
    if (!salaryCurrent.value || String(salaryCurrent.value.employee_id) !== String(e.id)) lookupSalary();
}

function notifySalarySaved() {
    const notify = useNotifyStore();
    notify.success(t("payroll.salaryAssigned"));
}

function openSalaryModal() {
    salaryError.value = null;
    closeRulesEditor();
    if (!employees.value.length) loadEmployees();
    salaryEmpId.value = "";
    salaryEmpQuery.value = "";
    salaryType.value = "monthly";
    salaryMonthly.value = "";
    salaryDaily.value = "";
    salaryHourly.value = "";
    salaryBasic.value = "";
    salaryDept.value = "";
    salaryEffective.value = "";
    salaryRules.value = [];
    salaryCopyFrom.value = "";
    salaryModal.setText(t("payroll.salaryAssign"), "");
    salaryModal.get().isShow = true;
    // No modal footer buttons: Batal + Tetapkan live inside the form.
}

// Edit rules of one salary row. Amounts and dates stay read-only.
const rulesRow = ref(null);
const rulesSaving = ref(false);
const rulesError = ref(null);

function openRulesEditor(row) {
    rulesError.value = null;
    rulesRow.value = row;
    salaryRules.value = (row.salary?.rules_json || []).map(normalizeLoadedRule);
    salaryModal.setText(t("payroll.salaryRulesEdit"), "");
    salaryModal.get().isShow = true;
}

function closeRulesEditor() {
    rulesRow.value = null;
    rulesError.value = null;
    salaryRules.value = [];
}

function closeSalaryModal() {
    closeRulesEditor();
    salaryModal.clear();
}

// Save edited rules of one row. Only rules_json is sent to the server.
async function handleSaveRules() {
    rulesError.value = null;
    if (!rulesRow.value?.salary) return;
    const rulesPayload = buildRulesPayload(salaryRules.value);
    rulesSaving.value = true;
    try {
        await payroll.updateSalaryRules(rulesRow.value.salary.id, rulesPayload);
        const notify = useNotifyStore();
        notify.success(t("payroll.salaryRulesSaved"));
        closeSalaryModal();
        await loadSalaryTable();
    } catch (e) {
        rulesError.value = e.message;
    } finally {
        rulesSaving.value = false;
    }
}

function salaryInitials(emp) {
    const a = (emp.first_name || emp.name || "?")[0] || "";
    const b = (emp.last_name || "")[0] || "";
    return (a + b).toUpperCase().slice(0, 2);
}
function salaryAvatar(emp) {
    const s = `${emp.first_name || ""}${emp.last_name || ""}`;
    let h = 0;
    for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) % 360;
    return `hsl(${140 + (h % 40) - 20} 70% 38%)`;
}

async function loadEmployees() {
    try {
        const r = await apiFetch(`${config.public.apiBase}/employee/list`, { method: 'GET', headers: auth.confHeaders() });
        if (r.code === 200) employees.value = (r.data || []).filter(x => String(x.status) === '0');
    } catch (e) { /* list stays empty */ }
}

async function lookupSalary() {
    salaryCurrent.value = null;
    salaryError.value = null;
    if (!salaryEmpId.value) return;
    try {
        salaryCurrent.value = await payroll.getSalary(salaryEmpId.value);
        if (salaryCurrent.value) {
            salaryType.value = salaryCurrent.value.salary_type || "monthly";
            if (salaryCurrent.value.department) salaryDept.value = salaryCurrent.value.department;
            const curRules = salaryCurrent.value.rules_json || [];
            salaryRules.value = (Array.isArray(curRules) ? curRules : []).map(normalizeLoadedRule);
        } else {
            salaryRules.value = [];
        }
    } catch (e) {
        salaryError.value = e.message;
    }
}

// Show every salary row, including old versions. The newest row comes first per employee.
function exportSalaryCsv() {
    const cell = csvCell;
    const fmtDate = (d) => {
        if (!d) return "";
        const m = String(d).match(/^(\d{4})-(\d{2})-(\d{2})/);
        return m ? `${m[3]}/${m[2]}/${m[1]}` : String(d);
    };
    const out = ["EM,ID,TYPE,EFFECTIVE_DATE,EMPLOYEE_NAME,MONTLY_RATE,DAILY_RATE,HOURLY_RATE", "RL,RULE_NAME,RULE_TYPE,RULE_FORMULA,IS_DAILY,,,"];
    for (const r of filteredSalaryTable.value) {
        if (!r.salary) continue;
        const s = r.salary;
        const name = [r.employee.first_name || r.employee.name, r.employee.last_name].filter(Boolean).join(" ");
        out.push(["EM", r.employee.id, (s.salary_type || "monthly").toUpperCase(), fmtDate(s.effective_date), name, s.monthly_rate, s.daily_salary, s.hourly_rate].map(cell).join(","));
        const rules = Array.isArray(s.rules_json) ? s.rules_json : [];
        for (const rl of rules) {
            const type = rl.deduct ? "SUB" : "ADD";
            const formula = rl.parameter || rl.function || "";
            const isDaily = rl.daily ? "1" : "0";
            out.push(["RL", rl.name, type, formula, isDaily, "", "", ""].map(cell).join(","));
        }
    }
    const csv = out.join("\n");
    const blob = new Blob([csv], { type: "text/csv" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `salary-export.csv`;
    a.click();
    URL.revokeObjectURL(a.href);
}
async function loadSalaryTable() {
    salaryTableLoading.value = true;
    try {
        if (!employees.value.length) await loadEmployees();
        const byId = {};
        for (const e of (employees.value || [])) byId[String(e.id)] = e;
        const all = await payroll.getAllSalaries();
        const rows = (all || []).map((s) => {
            const emp = byId[String(s.employee_id)] || { id: s.employee_id, first_name: s.employee_name || "?", last_name: "", department: s.department || "" };
            return { key: `s-${s.id}`, employee: { ...emp, department: s.department || emp.department }, salary: s };
        });
        // Employees without any row still show one placeholder line for quick add.
        for (const e of (employees.value || [])) {
            if (!rows.some((r) => String(r.employee.id) === String(e.id))) rows.push({ key: `e-${e.id}`, employee: e, salary: null });
        }
        salaryTable.value = rows;
    } catch (e) {
        salaryError.value = e.message;
    } finally {
        salaryTableLoading.value = false;
    }
}

function salaryDisplayFor(row) {
    const s = row.salary;
    if (!s) return "-";
    if ((s.salary_type || "monthly") === "daily") return fmtMoney(s.daily_salary);
    return fmtMoney(s.monthly_rate);
}


async function handleAssignSalary() {
    salaryError.value = null;
    const monthlyNum = salaryMonthly.value === "" ? null : parseInt(salaryMonthly.value);
    const dailyNum = salaryDaily.value === "" ? null : parseInt(salaryDaily.value);
    const hourlyNum = salaryHourly.value === "" ? null : parseInt(salaryHourly.value);
    const basicNum = salaryBasic.value === "" || salaryBasic.value == null
        ? (salaryType.value === "daily" ? dailyNum : monthlyNum)
        : parseInt(salaryBasic.value);
    if (!salaryEmpId.value || !salaryEffective.value || basicNum == null || Number.isNaN(basicNum)) {
        salaryError.value = t("payroll.salaryNeedsAll");
        return;
    }
    if (salaryType.value === "monthly" && (monthlyNum == null || Number.isNaN(monthlyNum))) {
        salaryError.value = t("payroll.salaryNeedsAll");
        return;
    }
    if (dailyNum == null || Number.isNaN(dailyNum) || hourlyNum == null || Number.isNaN(hourlyNum)) {
        salaryError.value = t("payroll.salaryNeedsAll");
        return;
    }
    salarySaving.value = true;
    try {
        // Keep BE payload fields stable: function carries the kind label,
        // parameter carries the formula, add/deduct mirror the combobox.
        const rulesPayload = buildRulesPayload(salaryRules.value);
        const payload = {
            employee_id: parseInt(salaryEmpId.value),
            salary_type: salaryType.value,
            monthly_rate: monthlyNum ?? 0,
            daily_salary: dailyNum,
            hourly_rate: hourlyNum,
            basic_salary: basicNum,
            department: salaryDept.value ? salaryDept.value.trim() || undefined : undefined,
            rules_json: rulesPayload,
            effective_date: salaryEffective.value,
        };
        await payroll.assignSalary(payload);
        notifySalarySaved();
        salaryMonthly.value = "";
        salaryDaily.value = "";
        salaryHourly.value = "";
        salaryBasic.value = "";
        salaryRules.value = [];
        salaryCopyFrom.value = "";
        await lookupSalary();
        await loadSalaryTable();
    } catch (e) {
        // The store keeps the backend i18n key; the demo mock bypasses apiFetch's
        // translate(), so localize here where the locale is in scope.
        salaryError.value = e.key ? t(e.key) : e.message;
    } finally {
        salarySaving.value = false;
    }
}

// Format dates for input
function toInputDate(d) {
    if (!d) return "";
    if (typeof d === "string") return d;
    if (d instanceof Date) {
        return d.toISOString().split("T")[0];
    }
    return "";
}

// Format dates for display: plain date stays date-only; timestamp shows date + time.
function displayDate(d) {
    if (!d) return "-";
    if (typeof d === "string" && /^\d{4}-\d{2}-\d{2}$/.test(d)) {
        const dt = new Date(d + "T00:00:00Z");
        return dt.toLocaleDateString('id-ID');
    }
    const dt = typeof d === "string" ? new Date(d) : d;
    if (isNaN(dt)) return typeof d === "string" ? d : "-";
    return dt.toLocaleDateString('id-ID') + " " + dt.toLocaleTimeString('id-ID', { hour: "2-digit", minute: "2-digit" });
}

const condEmpSuggest = ref(null);

function addConditionRow() {
    const notify = useNotifyStore();
    if (!newCondition.value.employee_id || !newCondition.value.work_date || !newCondition.value.condition || !String(newCondition.value.reason || "").trim()) {
        notify.error(t("payroll.condNeedsAll"));
        return;
    }
    const dup = dailyConditions.value.some((c) => String(c.employee_id) === String(newCondition.value.employee_id) && c.work_date === newCondition.value.work_date);
    if (dup) {
        notify.error(t("payroll.condDuplicate"));
        return;
    }
    const emp = employees.value.find((e) => String(e.id) === String(newCondition.value.employee_id));
    dailyConditions.value.push({
        employee_id: parseInt(newCondition.value.employee_id),
        employee_name: emp ? [emp.first_name || emp.name, emp.last_name].filter(Boolean).join(" ") : "",
        work_date: newCondition.value.work_date,
        condition: newCondition.value.condition,
        reason: String(newCondition.value.reason || "").trim(),
    });
    newCondition.value = { employee_id: "", work_date: null, condition: "", reason: "" };
    condEmpSuggest.value?.reset();
}

function removeCondition(idx) {
    dailyConditions.value.splice(idx, 1);
}

// Queue count + period label (report pattern: computed chips)
const queueCount = computed(() => dailyConditions.value.length);
const periodLabel = computed(() => {
    if (!periodStart.value || !periodEnd.value) return "";
    return `${displayDate(periodStart.value)} - ${displayDate(periodEnd.value)}`;
});

// Range from SimpleDatePicker (same as report page): ev carries startDate/endDate YYYY-MM-DD.
function onPayrollRange(ev) {
    if (!ev || !ev.startDate || !ev.endDate) return;
    periodStart.value = ev.startDate;
    periodEnd.value = ev.endDate;
    generateError.value = null;
    generateResult.value = null;
}

async function handleGenerate() {
    generateError.value = null;
    generateResult.value = null;

    if (!periodStart.value || !periodEnd.value) {
        generateError.value = t("payroll.needsDateRange");
        return;
    }

    const start = new Date(periodStart.value + "T00:00:00Z");
    const end = new Date(periodEnd.value + "T00:00:00Z");
    if (end < start) {
        generateError.value = t("payroll.errDateRange");
        return;
    }

    generating.value = true;
    generateResult.value = null;
    try {
        // Scope: exactly one of employee_id / department, else whole company.
        const scope = {};
        if (genScope.value === "employee" && genEmployeeId.value) scope.employee_id = Number(genEmployeeId.value);
        else if (genScope.value === "department" && genDepartment.value) scope.department = genDepartment.value;
        // reason required by BE valid_absence_reason guard; sent with each condition
        const payload = dailyConditions.value.map(({ employee_id, work_date, condition, reason }) => ({ employee_id, work_date, condition, reason }));
        const res = await payroll.generate(
            periodStart.value,
            periodEnd.value,
            payload.length > 0 ? payload : null,
            scope
        );
        if (res && res.code === 201) {
            const notify = useNotifyStore();
            // Success message may carry skipped names as **Name - Dept**; toast stays localized.
            const skipped = [];
            const cleanMsg = String(res.message || "").replace(/\*\*([^*]+)\*\*/g, (_, name) => {
                skipped.push(String(name));
                return "";
            }).replace(/\s{2,}/g, " ").trim();
            notify.success(cleanMsg ? t("payroll.generatedSuccess") + (skipped.length ? ": " + skipped.join(", ") : "") : t("payroll.generatedSuccess"));
            generateResult.value = {
                count: Array.isArray(res.data) ? res.data.length : 0,
                period_start: displayDate(periodStart.value),
                period_end: displayDate(periodEnd.value),
                skipped,
            };
            // Skipped names show only in the amber result card, not as red error.
            generateError.value = null;
            dailyConditions.value = [];
        } else {
            generateError.value = res?.message || t("payroll.generateFailed");
        }
    } catch (e) {
        console.error("Generate error:", e);
        generateError.value = e.message || t("payroll.generateFailed");
    } finally {
        generating.value = false;
    }
}

// DB stores the condition code (absence_reason), not a label: map the known
// codes through i18n and fall back to the raw value for legacy/free-text rows.
const CONDITION_KEYS = {
    present: 'payroll.conditionPresent',
    lupa_absen: 'payroll.conditionLupa_absen',
    cuti: 'payroll.conditionCuti',
    izin: 'payroll.conditionIzin',
    sakit: 'payroll.conditionSakit',
    absent: 'payroll.conditionAbsent',
    no_punch_in: 'payroll.conditionNoPunchIn',
    no_punch_out: 'payroll.conditionNoPunchOut',
};
function conditionLabel(raw) {
    const key = CONDITION_KEYS[String(raw || '').trim().toLowerCase()];
    return key ? t(key) : String(raw || '');
}

// Backend wraps employee who in **Name - Dept**; render bold, escape rest.
const generateErrorHtml = computed(() => {
    const raw = generateError.value || "";
    const who = [];
    const scrubbed = raw.replace(/\*\*([^*]+)\*\*/g, (_, name) => {
        who.push(String(name));
        return "\u0001" + (who.length - 1) + "\u0001";
    });
    const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    const strong = (n) => "<strong>" + esc(n) + "</strong>";
    const miss = /without salary|tanpa gaji/i.test(scrubbed);
    const head = t("payroll.missingSalaryHead", { count: who.length });
    const tail = t("payroll.missingSalaryTail");
    if (who.length && miss) {
        return esc(head) + " (" + who.map(strong).join(", ") + "). " + esc(tail);
    }
    // Rule engine gates have no i18n key (they carry a computed date), so match
    // the fixed english stem and localize the sentence around it.
    const negDay = scrubbed.match(/rule produced negative day total on ([\d-]+)/i);
    if (negDay) {
        return (who.length ? who.map(strong).join(", ") + ": " : "")
            + esc(t("payroll.ruleNegativeDay", { date: negDay[1] }));
    }
    if (/rule produced negative period total/i.test(scrubbed)) {
        return (who.length ? who.map(strong).join(", ") + ": " : "")
            + esc(t("payroll.ruleNegativePeriod"));
    }
    // Rule errors: BE already prefixes **Name - Dept**; bold who + rule 'x' name.
    const rich = esc(raw).replace(/\*\*([^*]+)\*\*/g, (_, n) => strong(n))
        .replace(/rule &#x27;([^&]+)&#x27;|rule '([^']+)'/g, (_, a, b) => "rule " + strong(a || b));
    return rich;
});

const slipPage = ref(0);
const SLIP_PAGE = 50;
const slipTotal = computed(() => payroll.payslipMeta.total || 0);

async function loadList() {
    listLoading.value = true;
    try {
        const items = await payroll.list(filterStart.value || null, filterEnd.value || null, {
            limit: SLIP_PAGE,
            offset: slipPage.value * SLIP_PAGE,
        });
        payslips.value = items || [];
    } catch (e) {
        console.error("List error:", e);
    } finally {
        listLoading.value = false;
    }
}

async function handleLock(payslipId) {
    try {
        await payroll.lock(payslipId);
        const notify = useNotifyStore();
        notify.success(t("payroll.locked"));
        await applyListFilter();
    } catch (e) {
        const notify = useNotifyStore();
        notify.error(e.message || t("payroll.lockFailed"));
    }
}

// Delete salary row: confirm first, server removes the version row.
const salaryDeleteModal = new SMData();
const salaryDeleteTarget = ref(null);

function askDeleteSalary(row) {
    salaryDeleteTarget.value = row;
    const e = row.employee || {};
    const empName = [e.first_name || e.name, e.last_name].filter(Boolean).join(" ") || "-";
    salaryDeleteModal.setText(t("payroll.salaryDeleteTitle"), t("payroll.salaryDeleteConfirm", {
        name: empName,
        date: row.salary ? displayDate(row.salary.effective_date) : "-",
    }));
    salaryDeleteModal.showOKCancel();
}

function cancelDeleteSalary() {
    salaryDeleteTarget.value = null;
    salaryDeleteModal.clear();
}

async function confirmDeleteSalary() {
    if (!salaryDeleteTarget.value || !salaryDeleteTarget.value.salary) { salaryDeleteModal.clear(); return; }
    const id = salaryDeleteTarget.value.salary.id;
    salaryDeleteTarget.value = null;
    salaryDeleteModal.clear();
    try {
        await payroll.removeSalary(id);
        const notify = useNotifyStore();
        notify.success(t("payroll.salaryDeleted"));
        await loadSalaryTable();
    } catch (e) {
        salaryError.value = e.message;
        const notify = useNotifyStore();
        notify.error(e.message || t("payroll.deleteFailed"));
    }
}

// Delete draft payslip: confirm first, then refresh the list.
const deleteModal = new SMData();
const deleteTarget = ref(null);
// Checked rows for bulk draft delete (ids only, locked rows excluded).
const checkedDrafts = ref([]);
const draftSlips = computed(() => payslips.value.filter((s) => s.status !== "locked"));

function askDeleteDraft(slip) {
    deleteTarget.value = slip;
    deleteModal.setText(t("payroll.deleteTitle"), t("payroll.deleteConfirm", {
        name: payslipEmployeeName(slip.employee_id),
        period: `${displayDate(slip.period_start)} - ${displayDate(slip.period_end)}`,
    }));
    deleteModal.showOKCancel();
}

function askDeleteDrafts(ids) {
    deleteTarget.value = ids;
    deleteModal.setText(t("payroll.deleteTitleBulk"), t("payroll.deleteConfirmBulk", {
        count: ids.length,
    }));
    deleteModal.showOKCancel();
}

function cancelDeleteDraft() {
    deleteTarget.value = null;
    deleteModal.clear();
}

async function confirmDeleteDraft() {
    if (!deleteTarget.value) { deleteModal.clear(); return; }
    // Bulk path: array of ids (locked rows already excluded).
    if (Array.isArray(deleteTarget.value)) {
        const ids = deleteTarget.value;
        deleteTarget.value = null;
        deleteModal.clear();
        try {
            const removed = await payroll.removeDrafts(ids);
            const notify = useNotifyStore();
            notify.success(t("payroll.deletedBulk", { count: removed.length }));
            checkedDrafts.value = [];
            await applyListFilter();
        } catch (e) {
            const notify = useNotifyStore();
            notify.error(e.message || t("payroll.deleteFailed"));
        }
        return;
    }
    const id = deleteTarget.value.id;
    deleteTarget.value = null;
    deleteModal.clear();
    try {
        await payroll.removeDraft(id);
        const notify = useNotifyStore();
        notify.success(t("payroll.deleted"));
        await applyListFilter();
    } catch (e) {
        const notify = useNotifyStore();
        notify.error(e.message || t("payroll.deleteFailed"));
    }
}

const filterStart = ref("");
// Bumped on Clear so the picker remounts empty instead of keeping the old
// range in its inputs.
const filterPickerKey = ref(0);
const filterEnd = ref("");

function onFilterRange(ev) {
    if (!ev || !ev.startDate || !ev.endDate) return;
    filterStart.value = ev.startDate;
    filterEnd.value = ev.endDate;
}

async function applyListFilter() {
    if (slipPage.value !== 0) slipPage.value = 0;
    await loadList();
    // Drop checked ids that are gone or no longer draft.
    const draftIds = new Set(payslips.value.filter((s) => s.status !== "locked").map((s) => s.id));
    checkedDrafts.value = checkedDrafts.value.filter((id) => draftIds.has(id));
}

function clearListFilter() {
    filterStart.value = "";
    filterEnd.value = "";
    filterPickerKey.value += 1;
    slipPage.value = 0;
    loadList();
}

function goSlipPage(next) {
    if (next === slipPage.value) return;
    slipPage.value = next;
    loadList();
}

function goAbsPage(next) {
    if (next === absPage.value) return;
    absPage.value = next;
    loadAbsences();
}

// Daily absences: pending missed-punch approvals
const absences = ref([]);
const absLoading = ref(false);
const absError = ref(null);
const absStatus = ref("pending_verification");
// Absence filter: pick a person by name/department; an id alone is unreadable.
const absEmpId = ref(null);
function pickAbsEmp(e) {
    absEmpId.value = e.id;
    absPage.value = 0;
    loadAbsences();
}
function clearAbsEmp() {
    absEmpId.value = null;
    absPage.value = 0;
    loadAbsences();
}
const absPage = ref(0);
const ABS_PAGE = 50;
const absTotal = computed(() => payroll.absenceMeta.total || 0);

async function loadAbsences() {
    absLoading.value = true;
    absError.value = null;
    try {
        absences.value = await payroll.listDailyAbsences(absStatus.value === "all" ? null : absStatus.value, {
            limit: ABS_PAGE,
            offset: absPage.value * ABS_PAGE,
            employeeId: absEmpId.value,
        });
        if (!absences.value.length && absPage.value > 0 && absTotal.value > 0) {
            absPage.value = Math.max(0, Math.ceil(absTotal.value / ABS_PAGE) - 1);
        }
    } catch (e) {
        absError.value = e.message;
    } finally {
        absLoading.value = false;
    }
}

async function handleApproveAbsence(id) {
    try {
        await payroll.approveDailyAbsence(id);
        const notify = useNotifyStore();
        notify.success(t("payroll.absApproved"));
        await loadAbsences();
    } catch (e) {
        const notify = useNotifyStore();
        notify.error(e.message || t("payroll.absApproveFailed"));
    }
}

// Rejecting an absence is destructive and irreversible, so it goes through
// the shared confirm modal instead of running on the first click. Approve
// keeps a solid emerald button, reject gets an outlined red one so the
// pair is visually weighted and harder to trigger by accident.
const rejectModal = new SMData();
const rejectTarget = ref(null);

function askRejectAbsence(absence) {
    rejectTarget.value = absence?.id ?? absence;
    rejectModal.setText(t("payroll.absRejectTitle"), t("payroll.absRejectConfirm", {
        name: absenceEmployeeName(absence?.employee_id),
        date: displayDate(absence?.absence_date),
    }));
    rejectModal.showOKCancel();
}

function cancelRejectAbsence() {
    rejectTarget.value = null;
    rejectModal.clear();
}

async function confirmRejectAbsence() {
    const id = rejectTarget.value;
    rejectTarget.value = null;
    rejectModal.clear();
    if (!id) return;
    try {
        await payroll.rejectDailyAbsence(id);
        const notify = useNotifyStore();
        notify.success(t("payroll.absRejected"));
        await loadAbsences();
    } catch (e) {
        const notify = useNotifyStore();
        notify.error(e.message || t("payroll.absRejectFailed"));
    }
}

// Allow generate only if user has payroll Edit permission
const canGenerate = computed(() => auth.hasActionAccess("payroll", "can_add"));
const canView = computed(() => auth.hasActionAccess("payroll", "can_view"));
const canEdit = computed(() => auth.hasActionAccess("payroll", "can_edit"));
const canDelete = computed(() => auth.hasActionAccess("payroll", "can_delete"));

// Absence table: BE returns employee_id only, resolve display name from employees list
function absenceEmployeeName(id) {
    const emp = employees.value.find((e) => String(e.id) === String(id));
    if (!emp) return `#${id}`;
    return [emp.first_name || emp.name, emp.last_name].filter(Boolean).join(" ") || `#${id}`;
}

// Payslip rows: BE returns employee_id only, resolve name + dept from employees list
function payslipEmployee(id) {
    return employees.value.find((e) => String(e.id) === String(id)) || null;
}
function payslipEmployeeName(id) {
    const emp = payslipEmployee(id);
    if (!emp) return `#${id}`;
    return [emp.first_name || emp.name, emp.last_name].filter(Boolean).join(" ") || `#${id}`;
}
function payslipEmployeeDept(id) {
    const emp = payslipEmployee(id);
    return (emp && (emp.department || emp.employee_department)) || "-";
}
// Avatar helpers expect an employee object; payslip rows only carry employee_id.
function slipAvatarEmp(id) {
    return payslipEmployee(id) || { first_name: "?", last_name: "" };
}

onMounted(() => { loadEmployees(); });

</script>

<template>
    <div v-if="!canView" class="min-h-[60vh] flex items-center justify-center text-center">
        <div class="text-text-muted">{{ $t("nav.noAccess") }}</div>
    </div>
    <div v-else class="flex flex-col gap-4">
        <!-- Global header above tab menu -->
        <div class="flex flex-col sm:flex-row sm:items-center gap-3">
            <div class="min-w-0">
                <h1 class="display text-[20px] text-text">{{ $t("payroll.title") }}</h1>
                <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
                    <span class="chip chip-quiet">
                        <span class="w-1.5 h-1.5 rounded-full"
                            :class="generating ? 'bg-amber-500 data-tick' : 'bg-primary'"></span>
                        {{ queueCount }} {{ $t("payroll.queuedUnit") }}
                    </span>
                    <span class="chip chip-quiet">
                        {{ tab === 'generate' ? $t('payroll.generate') : tab === 'list' ? $t('payroll.listTitle') : tab
                            === 'salary' ? $t('payroll.salaryTab') : $t('payroll.absenceTab') }}
                    </span>
                </div>
            </div>
        </div>
        <!-- Tabs -->
        <div class="panel p-1.5 flex flex-wrap gap-1.5">
            <button @click="tab = 'generate'; loadEmployees()" type="button"
                :class="tab === 'generate' ? 'btn-primary' : 'btn-quiet'" class="btn flex-1 sm:flex-none">
                {{ $t("payroll.generate") }}
            </button>
            <button @click="tab = 'list'; loadEmployees(); loadList()" type="button"
                :class="tab === 'list' ? 'btn-primary' : 'btn-quiet'" class="btn flex-1 sm:flex-none">
                {{ $t("payroll.listTitle") }}
            </button>
            <button @click="tab = 'salary'; loadEmployees(); loadSalaryTable()" type="button"
                :class="tab === 'salary' ? 'btn-primary' : 'btn-quiet'" class="btn flex-1 sm:flex-none">
                {{ $t("payroll.salaryTab") }}
            </button>
            <button v-if="canEdit" @click="tab = 'absence'; loadEmployees(); loadAbsences()" type="button"
                :class="tab === 'absence' ? 'btn-primary' : 'btn-quiet'" class="btn flex-1 sm:flex-none">
                {{ $t("payroll.absenceTab") }}
            </button>
        </div>

        <!-- Generate Tab -->
        <div v-show="tab === 'generate'" class="space-y-4">
            <div class="grid gap-4 lg:grid-cols-2">
                <!-- Card1: period + generate -->
                <div class="panel p-3 sm:p-4 space-y-3">
                    <div class="card-label">{{ $t("payroll.periodTitle") }}</div>
                    <SimpleDatePicker :use-utc="true" :start-date="periodStart || ''" :end-date="periodEnd || ''"
                        :init-start-date="format.stamp_to_naive_date()" :init-end-date="format.stamp_to_naive_date()"
                        v-on:date-range-selected="onPayrollRange" />

                    <!-- Scope: all employees, one person, or one department -->
                    <div>
                        <label class="label">
                            {{ $t("payroll.genScope") }}
                        </label>
                        <select v-model="genScope" class="field">
                            <option value="all">{{ $t("payroll.genScopeAll") }}</option>
                            <option value="employee">{{ $t("payroll.genScopeEmployee") }}</option>
                            <option value="department">{{ $t("payroll.genScopeDepartment") }}</option>
                        </select>
                    </div>
                    <div v-if="genScope === 'employee'">
                        <label class="label">
                            {{ $t("payroll.genEmployee") }}
                        </label>
                        <input v-model="genEmpQuery" @input="onGenEmpInput" @change="matchGenEmp" @focus="onGenEmpFocus"
                            @blur="genEmpOpen = false" @keydown="onGenEmpKey" type="text" autocomplete="off"
                            :placeholder="$t('payroll.genPickEmployee')" class="field" />
                        <ul v-if="genEmpOpen && genEmpFilter.length"
                            class="mt-1 max-h-44 overflow-auto rounded-xl border border-border bg-surface shadow-lg">
                            <li v-for="(e, i) in genEmpFilter" :key="e.id" @mousedown.prevent="pickGenEmp(e)"
                                :class="i === genEmpIdx ? 'bg-surface-sunken is-active' : ''"
                                class="px-3 py-2 text-[13px] text-text cursor-pointer hover:bg-surface-raised">
                                <span class="block font-semibold truncate">{{ [e.first_name || e.name,
                                    e.last_name].filter(Boolean).join(' ') }}</span>
                                <span class="block mono text-[10px] text-text-muted truncate mt-0.5">{{ e.department ||
                                    '-' }}</span>
                            </li>
                        </ul>
                        <div v-if="genEmpOpen && !genEmpFilter.length"
                            class="mt-1 px-3 py-2 rounded-lg border border-border bg-surface-raised/50 text-xs text-text-muted">
                            {{ $t("payroll.salaryEmpEmpty") }}
                        </div>
                    </div>
                    <div v-if="genScope === 'department'">
                        <label class="label">
                            {{ $t("payroll.genDepartment") }}
                        </label>
                        <input v-model="genDeptQuery" @input="onGenDeptInput" @change="matchGenDept"
                            @focus="onGenDeptFocus" @blur="genDeptOpen = false" @keydown="onGenDeptKey" type="text"
                            autocomplete="off" :placeholder="$t('payroll.genPickDepartment')" class="field" />
                        <ul v-if="genDeptOpen && genDeptFilter.length"
                            class="mt-1 max-h-44 overflow-auto rounded-xl border border-border bg-surface shadow-lg">
                            <li v-for="(d, i) in genDeptFilter" :key="d" @mousedown.prevent="pickGenDept(d)"
                                :class="i === genDeptIdx ? 'bg-surface-sunken is-active' : ''"
                                class="px-3 py-2.5 min-h-[44px] text-[13px] text-text cursor-pointer hover:bg-surface-raised">
                                {{ d }}</li>
                        </ul>
                        <div v-if="genDeptOpen && !genDeptFilter.length"
                            class="mt-1 px-3 py-2 rounded-lg border border-border bg-surface-raised/50 text-xs text-text-muted">
                            {{ $t("payroll.salaryEmpEmpty") }}
                        </div>
                    </div>

                    <!-- Error: missing-salary names render bold via inner HTML. -->
                    <div v-if="generateError"
                        class="rounded-lg bg-danger-bg border border-red-200 px-3 py-2 text-[12.5px] text-danger"
                        v-html="generateErrorHtml"></div>

                    <!-- Submit -->
                    <button @click="handleGenerate" :disabled="generating || !canGenerate" type="button"
                        class="btn btn-primary">
                        {{ generating ? $t("payroll.generating") : $t("payroll.generate") }}
                    </button>

                    <!-- Inline result: report + employee pattern, stays on generate tab -->
                    <div v-if="generateResult"
                        class="rounded-xl border border-border bg-surface-raised/60 p-3 space-y-2">
                        <div class="flex flex-wrap items-center gap-2 mono text-[10px]">
                            <span class="chip chip-emerald">{{ generateResult.count }} {{ $t("payroll.generatedUnit")
                                }}</span>
                            <span class="text-text-muted">{{ generateResult.period_start }} - {{
                                generateResult.period_end }}</span>
                            <button @click="tab = 'list'; loadEmployees(); loadList()" type="button"
                                class="btn btn-quiet btn-sm !min-h-[44px] sm:!min-h-[36px] ml-auto">{{
                                $t("payroll.viewList") }}</button>
                        </div>
                        <div v-if="generateResult.skipped && generateResult.skipped.length"
                            class="mt-2 rounded-lg bg-warn-bg border border-amber-200 px-3 py-2 text-[12px] text-warn">
                            {{ $t("payroll.skippedNoSalary") }}: <strong>{{ generateResult.skipped.join(", ")
                                }}</strong>
                        </div>
                    </div>
                </div>

                <!-- Card2: condition queue form + list -->
                <div class="panel p-3 sm:p-4 space-y-2">
                    <div class="flex items-center justify-between">
                        <div class="card-label">
                            {{ $t("payroll.conditions") }}
                        </div>
                        <p class="text-[11.5px] text-text-muted">{{ $t("payroll.conditionsHint") }}</p>
                    </div>
                    <div v-if="dailyConditions.length > 0"
                        class="bg-surface-raised rounded-lg overflow-hidden hidden md:block">
                        <table class="w-full text-sm">
                            <thead>
                                <tr>
                                    <th class="th">
                                        {{ $t("payroll.colEmployee") }}
                                    </th>
                                    <th class="th">
                                        {{ $t("payroll.colDate") }}
                                    </th>
                                    <th class="th">
                                        {{ $t("payroll.colCondition") }}
                                    </th>
                                    <th class="th">
                                        {{ $t("payroll.colReason") }}
                                    </th>
                                    <th class="th w-8"></th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr v-for="(cond, idx) in dailyConditions" :key="idx"
                                    class="hover:bg-surface-sunken/40">
                                    <td class="td">{{ cond.employee_id }}<span v-if="cond.employee_name"> - {{
                                            cond.employee_name }}</span></td>
                                    <td class="td">{{ displayDate(cond.work_date) }}</td>
                                    <td class="td mono">{{ cond.condition }}</td>
                                    <td class="td !text-text-muted max-w-[180px] truncate">{{ cond.reason || "-" }}</td>
                                    <td class="td">
                                        <button @click="removeCondition(idx)" type="button"
                                            :title="$t('payroll.removeCondition')"
                                            class="btn btn-quiet btn-sm !min-h-[44px] sm:!min-h-[36px] !px-2 text-danger">
                                            <font-awesome :icon="faXmark" class="w-3 h-3" />
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    <!-- Condition queue mobile cards: same rows as the desktop table -->
                    <div v-if="dailyConditions.length > 0" class="md:hidden space-y-2">
                        <div v-for="(cond, idx) in dailyConditions" :key="`cond-m-${idx}`"
                            class="rounded-xl border border-border bg-surface p-3 flex items-start justify-between gap-2">
                            <div class="min-w-0">
                                <p class="text-xs font-bold text-text">{{ cond.employee_name || cond.employee_id }}</p>
                                <p class="mono text-[10px] text-text-muted mt-0.5">{{ displayDate(cond.work_date) }}</p>
                                <p class="mono text-[10px] text-text mt-0.5">{{ cond.condition }}<span
                                        v-if="cond.reason"> - {{ cond.reason }}</span></p>
                            </div>
                            <button @click="removeCondition(idx)" type="button" :title="$t('payroll.removeCondition')"
                                class="btn btn-quiet btn-sm !min-h-[44px] sm:!min-h-[36px] !px-2 shrink-0 text-danger">
                                <font-awesome :icon="faXmark" class="w-3 h-3" />
                            </button>
                        </div>
                    </div>

                    <!-- Add condition row -->
                    <div class="flex flex-col sm:flex-row sm:flex-wrap gap-2">
                        <div class="relative w-full sm:flex-1 sm:min-w-[280px]">
                            <input v-model="empQuery" type="text" class="field w-full text-xs has-icon-right"
                                :placeholder="$t('payroll.colEmployee')" autocomplete="off" role="combobox"
                                aria-autocomplete="list" :aria-expanded="empOpen" @focus="empOpen = true"
                                @input="onEmpInput" @keydown.esc.prevent="empOpen = false"
                                @blur="setTimeout(() => { empOpen = false; }, 160)" />
                            <font-awesome v-if="empQuery" :icon="faXmark"
                                class="absolute right-3 top-1/2 -translate-y-1/2 w-3 h-3 text-text-muted pointer-events-none" />
                            <ul v-if="empOpen && empMatches.length"
                                class="absolute z-20 mt-1 w-full min-w-[300px] max-h-56 overflow-auto rounded-xl border border-border bg-surface shadow-lg p-1">
                                <li v-for="e in empMatches" :key="e.id">
                                    <button type="button"
                                        class="w-full text-left px-2.5 py-2 rounded-lg text-xs hover:bg-surface-raised"
                                        @mousedown.prevent="pickConditionEmployee(e)">
                                        <span class="block font-semibold truncate">
                                            {{ [e.first_name || e.name, e.last_name].filter(Boolean).join(' ') }}
                                        </span>
                                        <span class="block mono text-[10px] text-text-muted truncate mt-0.5">
                                            {{ e.department || '-' }}
                                        </span>
                                    </button>
                                </li>
                            </ul>
                            <p v-if="empOpen && !empMatches.length"
                                class="absolute z-20 mt-1 w-full rounded-xl border border-border bg-surface shadow-lg px-2.5 py-2 text-[11px] text-text-muted">
                                {{ $t('employee.noData') }}
                            </p>
                        </div>
                        <input v-model="newCondition.work_date" type="date"
                            class="field w-full sm:!w-36 shrink-0 text-xs" />
                        <select v-model="newCondition.condition" class="field w-full sm:!w-44 shrink-0 text-xs">
                            <option value="">{{ $t("payroll.colCondition") }}</option>
                            <option value="present">{{ $t("payroll.conditionPresent") }}</option>
                            <option value="lupa_absen">{{ $t("payroll.conditionLupa_absen") }}</option>
                            <option value="cuti">{{ $t("payroll.conditionCuti") }}</option>
                            <option value="izin">{{ $t("payroll.conditionIzin") }}</option>
                            <option value="sakit">{{ $t("payroll.conditionSakit") }}</option>
                            <option value="absent">{{ $t("payroll.conditionAbsent") }}</option>
                        </select>
                        <input v-model="newCondition.reason" :placeholder="$t('payroll.colReason')"
                            class="field flex-1 min-w-0 text-xs" />
                        <button @click="addConditionRow" type="button"
                            class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px] shrink-0">
                            {{ $t("payroll.addCondition") }}
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- List Tab -->
        <div v-show="tab === 'list'" class="space-y-4">
            <!-- Filter card above payslip table -->
            <div class="panel p-3 sm:p-4 space-y-2">
                <div class="card-label">{{ $t("payroll.payslipFilter") }}</div>
                <div class="flex flex-col sm:flex-row gap-2">
                    <SimpleDatePicker :key="filterPickerKey" :use-utc="true" :start-date="filterStart || ''"
                        :end-date="filterEnd || ''" v-on:date-range-selected="onFilterRange" />
                    <button @click="applyListFilter" type="button"
                        class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                        {{ $t("payroll.filterApply") }}
                    </button>
                    <button @click="clearListFilter" type="button"
                        class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                        {{ $t("payroll.filterClear") }}
                    </button>
                </div>
            </div>
            <!-- Payslip table: same layout as salary table -->
            <div class="panel overflow-hidden flex flex-col">
                <div class="panel-head">
                    <span class="inline-flex items-center gap-2 min-w-0">
                        <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                        <span class="truncate">{{ $t("payroll.listTitle") }}</span>
                        <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted hidden sm:inline">{{
                            $t("payroll.countShown", { shown: payslips.length }) }}</span>
                    </span>
                    <div class="flex items-center gap-1.5 shrink-0">
                        <span v-if="checkedDrafts.length" class="chip chip-danger">{{ $t("payroll.selectedCount", {
                            count: checkedDrafts.length }) }}</span>
                        <button v-if="checkedDrafts.length && canDelete" @click="askDeleteDrafts([...checkedDrafts])"
                            type="button" class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px]">
                            {{ $t("payroll.deleteSelected") }}
                        </button>
                        <button @click="loadEmployees(); loadList()" type="button"
                            class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                            {{ $t("payroll.salaryReload") }}
                        </button>
                    </div>
                </div>
                <div v-if="listLoading" class="text-center py-6 text-[13px] text-text-muted">
                    {{ $t("payroll.loading") }}
                </div>
                <div v-else-if="payslips.length === 0" class="hidden md:block">
                    <div class="state">
                        <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
                        <p class="state-title">{{ $t("payroll.noPayslips") }}</p>
                        <button v-if="canGenerate" @click="tab = 'generate'"
                            class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{ $t("payroll.goGenerate")
                            }}</button>
                    </div>
                </div>
                <div v-else class="table-shell hidden md:block max-h-[56vh]">
                    <table class="w-full min-w-[760px] border-collapse">
                        <thead class="sticky top-0 z-10">
                            <tr>
                                <th class="th w-[4%]">
                                    <input v-if="draftSlips.length && canDelete" type="checkbox"
                                        :checked="checkedDrafts.length > 0 && checkedDrafts.length === draftSlips.length"
                                        :indeterminate="checkedDrafts.length > 0 && checkedDrafts.length < draftSlips.length"
                                        @change="checkedDrafts = $event.target.checked ? draftSlips.map((s) => s.id) : []"
                                        class="w-4 h-4 accent-emerald-600 cursor-pointer" />
                                </th>
                                <th class="th w-[28%]">{{ $t("payroll.colEmployee") }}</th>
                                <th class="th w-[26%]">{{ $t("payroll.colPeriod") }}</th>
                                <th class="th w-[14%]">{{ $t("payroll.colStatus") }}</th>
                                <th class="th w-[20%]">{{ $t("payroll.colCreated") }}</th>
                                <th class="th !text-right w-[12%]">{{ $t("payroll.colActions") }}</th>
                            </tr>
                        </thead>
                        <tbody class="bg-surface">
                            <tr v-for="(slip, idx) in payslips" :key="slip.id" class="even:bg-surface-raised/25">
                                <td class="td">
                                    <input v-if="slip.status !== 'locked' && canDelete" v-model="checkedDrafts"
                                        :value="slip.id" type="checkbox"
                                        class="w-4 h-4 accent-emerald-600 cursor-pointer" />
                                </td>
                                <td class="td">
                                    <div class="flex items-center gap-3">
                                        <div class="w-9 h-9 rounded-xl flex items-center justify-center text-white mono text-[11px] font-extrabold shrink-0 shadow-sm ring-1 ring-border-strong"
                                            :style="{ background: salaryAvatar(slipAvatarEmp(slip.employee_id)) }">{{
                                            salaryInitials(slipAvatarEmp(slip.employee_id)) }}</div>
                                        <div class="min-w-0">
                                            <p class="text-[13px] font-bold text-text leading-none truncate">{{
                                                payslipEmployeeName(slip.employee_id) }}</p>
                                            <p class="mono text-[10px] tracking-wide text-text-muted mt-1 truncate">{{
                                                payslipEmployeeDept(slip.employee_id) }}</p>
                                        </div>
                                    </div>
                                </td>
                                <td class="td mono text-[11.5px] !text-text-muted">
                                    {{ displayDate(slip.period_start) }} - {{ displayDate(slip.period_end) }}
                                </td>
                                <td class="td">
                                    <span v-if="slip.status === 'locked'" class="chip chip-warn">{{ $t("payroll.locked")
                                        }}</span>
                                    <span v-else class="chip chip-ok">{{ slip.status === 'draft' ?
                                        $t("payroll.slipDraft") : slip.status }}</span>
                                </td>
                                <td class="td mono text-[11.5px] !text-text-muted">
                                    {{ displayDate(slip.created_at) }}
                                </td>
                                <td class="td !text-right whitespace-nowrap">
                                    <button @click="openDetail(slip.id)" type="button"
                                        class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                                        {{ $t("payroll.detail") }}
                                    </button>
                                    <button v-if="slip.status !== 'locked' && canDelete" @click="askDeleteDraft(slip)"
                                        type="button" class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px] ml-1">
                                        {{ $t("common.delete") }}
                                    </button>
                                    <button v-if="slip.status !== 'locked' && canGenerate" @click="handleLock(slip.id)"
                                        type="button"
                                        class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px] ml-1">
                                        {{ $t("payroll.lock") }}
                                    </button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
                <!-- Mobile cards mirror salary cards -->
                <div class="md:hidden p-3 space-y-3 max-h-[52vh] overflow-auto bg-surface rounded-b-2xl"
                    v-if="!listLoading && payslips.length > 0">
                    <div v-for="slip in payslips" :key="`m-${slip.id}`"
                        class="rounded-xl border border-border bg-surface p-3.5">
                        <div class="flex items-start justify-between gap-3">
                            <div class="flex items-center gap-2.5">
                                <input v-if="slip.status !== 'locked' && canDelete" v-model="checkedDrafts"
                                    :value="slip.id" type="checkbox"
                                    class="w-4 h-4 accent-emerald-600 cursor-pointer shrink-0 mt-2" />
                                <div class="w-9 h-9 rounded-xl flex items-center justify-center text-white mono text-xs font-extrabold"
                                    :style="{ background: salaryAvatar(slipAvatarEmp(slip.employee_id)) }">{{
                                        salaryInitials(slipAvatarEmp(slip.employee_id)) }}</div>
                                <div>
                                    <p class="text-sm font-bold text-text leading-none">{{
                                        payslipEmployeeName(slip.employee_id) }}</p>
                                    <p class="mono text-[10px] text-text-muted mt-1">{{
                                        payslipEmployeeDept(slip.employee_id) }}</p>
                                </div>
                            </div>
                            <span class="chip" :class="slip.status === 'locked' ? 'chip-warn' : 'chip-ok'">{{
                                slip.status === 'locked' ? $t("payroll.locked") : slip.status === 'draft' ?
                                $t("payroll.slipDraft") : slip.status }}</span>
                        </div>
                        <div class="mt-3 grid grid-cols-2 gap-2 mono text-xs">
                            <div class="rounded-lg bg-canvas border border-border px-2.5 py-2"><span
                                    class="text-text-muted">{{ $t("payroll.colPeriod") }}</span><span
                                    class="block font-bold text-text">{{ displayDate(slip.period_start) }} - {{
                                    displayDate(slip.period_end) }}</span></div>
                            <div class="rounded-lg bg-canvas border border-border px-2.5 py-2"><span
                                    class="text-text-muted">{{ $t("payroll.colCreated") }}</span><span
                                    class="block font-bold text-text">{{ displayDate(slip.created_at) }}</span></div>
                        </div>
                        <div class="mt-2 flex justify-end gap-1.5">
                            <button @click="openDetail(slip.id)" type="button"
                                class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{ $t("payroll.detail")
                                }}</button>
                            <button v-if="slip.status !== 'locked' && canDelete" @click="askDeleteDraft(slip)"
                                type="button" class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px]">{{
                                $t("common.delete") }}</button>
                            <button v-if="slip.status !== 'locked' && canGenerate" @click="handleLock(slip.id)"
                                type="button" class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{
                                $t("payroll.lock") }}</button>
                        </div>
                    </div>
                </div>
                <div v-if="!listLoading && payslips.length === 0" class="md:hidden">
                    <div class="state">
                        <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
                        <p class="state-title">{{ $t("payroll.noPayslips") }}</p>
                        <button v-if="canGenerate" @click="tab = 'generate'"
                            class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{ $t("payroll.goGenerate")
                            }}</button>
                    </div>
                </div>
                <TablePager :page="slipPage" :page-size="SLIP_PAGE" :total="slipTotal" @update:page="goSlipPage" />
            </div>
        </div>

        <!-- Detail Modal -->
        <SimpleModal :options="detailModal.get()" @cancel="closeDetail" v-show="detailModal.isShow()">
            <div class="space-y-3 max-w-lg">
                <div v-if="detailLoading" class="text-sm text-text-muted">{{ $t("payroll.loading") }}</div>
                <div v-else-if="detailError"
                    class="rounded-lg bg-danger-bg border border-red-200 px-3 py-2 text-[12.5px] text-danger">
                    {{ detailError }}
                </div>
                <div v-else-if="detailSlip" class="space-y-3">
                    <div class="text-sm font-semibold text-text">
                        {{ payslipEmployeeName(detailSlip.employee_id) }} <span
                            class="mono text-[11px] font-normal text-text-muted">{{
                                payslipEmployeeDept(detailSlip.employee_id) }}</span> · {{
                                displayDate(detailSlip.period_start) }} - {{ displayDate(detailSlip.period_end) }}
                    </div>
                    <div class="grid grid-cols-2 gap-2 text-sm">
                        <div class="text-text-muted">{{ $t("payroll.dRegular") }}</div>
                        <div class="text-right font-mono">{{ fmtMoney(detailSlip.regular_days_pay) }}</div>
                        <div class="text-text-muted">{{ $t("payroll.dOt") }}</div>
                        <div class="text-right font-mono">{{ fmtMoney(detailSlip.ot_pay) }}</div>
                        <div class="text-text-muted">{{ $t("payroll.dHoliday") }}</div>
                        <div class="text-right font-mono">{{ fmtMoney(detailSlip.holiday_premium_pay) }}</div>
                        <div class="text-text-muted">{{ $t("payroll.dLeave") }}</div>
                        <div class="text-right font-mono">{{ fmtMoney(detailSlip.paid_leave_pay) }}</div>
                        <div class="font-semibold text-text">{{ $t("payroll.dGross") }}</div>
                        <div class="text-right font-mono font-semibold">{{ fmtMoney(detailSlip.gross_salary) }}</div>
                        <div class="text-text-muted">{{ $t("payroll.dDeduct") }}</div>
                        <div class="text-right font-mono">{{ fmtMoney(detailSlip.total_deductions) }}</div>
                        <div class="font-semibold text-text">{{ $t("payroll.dNet") }}</div>
                        <div class="text-right font-mono font-semibold">{{ fmtMoney(detailSlip.net_salary) }}</div>
                    </div>
                    <div v-if="detailSlip.lines && detailSlip.lines.length"
                        class="bg-surface-raised rounded-lg overflow-hidden max-h-48 overflow-y-auto">
                        <table class="w-full text-xs">
                            <tbody>
                                <tr v-for="l in detailSlip.lines" :key="l.id" class="hover:bg-surface-sunken/40">
                                    <td class="td mono">{{ l.work_date }}</td>
                                    <td class="td !text-right mono">{{ fmtMoney(l.daily_pay) }}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                    <button @click="exportCsv" type="button" class="btn btn-primary">
                        {{ $t("payroll.exportCsv") }}
                    </button>
                </div>
            </div>
        </SimpleModal>

        <!-- Delete Draft Confirm -->
        <SimpleModal :options="deleteModal.get()" @ok="confirmDeleteDraft" @cancel="cancelDeleteDraft"
            v-show="deleteModal.isShow()">
        </SimpleModal>

        <!-- Delete Salary Confirm -->
        <SimpleModal :options="salaryDeleteModal.get()" @ok="confirmDeleteSalary" @cancel="cancelDeleteSalary"
            v-show="salaryDeleteModal.isShow()">
        </SimpleModal>

        <!-- Salary Tab -->
        <div v-show="tab === 'salary'" class="space-y-4">
            <!-- Salary filter -->
            <div class="panel p-3 sm:p-4 space-y-3 relative z-10">
                <div class="flex flex-wrap items-center gap-1.5">
                    <span class="card-label">{{ $t("payroll.salaryFilter") }}</span>
                    <div class="flex flex-wrap items-center gap-1.5 lg:ml-auto">
                        <span class="card-label hidden sm:inline">{{ $t('report.quick') }}</span>
                        <button @click="salaryMode = 'all'" type="button"
                            class="btn btn-sm !min-h-[44px] sm:!min-h-[36px]"
                            :class="salaryMode === 'all' ? 'btn-primary' : 'btn-secondary'">
                            {{ $t("payroll.salaryAll") }}
                        </button>
                        <button @click="salaryMode = 'newest'" type="button"
                            class="btn btn-sm !min-h-[44px] sm:!min-h-[36px]"
                            :class="salaryMode === 'newest' ? 'btn-primary' : 'btn-secondary'">
                            {{ $t("payroll.salaryNewest") }}
                        </button>
                        <button @click="salaryMode = 'old'" type="button"
                            class="btn btn-sm !min-h-[44px] sm:!min-h-[36px]"
                            :class="salaryMode === 'old' ? 'btn-primary' : 'btn-secondary'">
                            {{ $t("payroll.salaryOld") }}
                        </button>
                    </div>
                </div>
                <!-- Row 2: employee filter + clear -->
                <div class="flex gap-2 items-center">
                    <div class="relative flex-1">
                        <input v-model="salaryFilterQuery" @input="onSalaryFilterInput" @focus="onSalaryFilterFocus"
                            @blur="salaryFilterOpen = false" @keydown="onSalaryFilterKey" type="text" autocomplete="off"
                            :placeholder="$t('payroll.colEmployee')" class="field has-icon-left" />
                        <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted"
                            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                            <path d="M21 21l-4.2-4.2" />
                            <circle cx="11" cy="11" r="6" />
                        </svg>
                        <ul v-if="salaryFilterOpen && salaryFilterList.length"
                            class="absolute z-[9999] mt-1 max-h-44 w-full overflow-auto rounded-xl border border-border bg-surface shadow-lg">
                            <li v-for="(e, i) in salaryFilterList" :key="e.id"
                                @mousedown.prevent="salaryFilterQuery = salaryEmpLabel(e); salaryFilterEmpId = e.id; salaryFilterOpen = false"
                                :class="i === salaryFilterIdx ? 'bg-surface-sunken' : ''"
                                class="px-3 py-2 text-[13px] text-text cursor-pointer hover:bg-surface-raised">
                                <span class="block font-semibold truncate">{{ [e.first_name || e.name,
                                    e.last_name].filter(Boolean).join(' ') }}</span>
                                <span class="block mono text-[10px] text-text-muted truncate mt-0.5">{{ e.department ||
                                    '-' }}</span>
                            </li>
                        </ul>
                    </div>
                    <button @click="salaryFilterQuery = ''; salaryFilterEmpId = null; salaryFilterOpen = false" type="button"
                        class="btn btn-secondary">
                        {{ $t("common.clear") }}
                    </button>
                </div>
            </div>
            <!-- Salary import -->
            <div class="panel p-3 sm:p-4 space-y-2">
                <div class="card-label">{{ $t("payroll.salaryImportTitle") }}</div>
                <div class="flex flex-col sm:flex-row gap-2 items-center">
                    <input type="file" accept=".csv" @change="importFile = $event.target.files[0]"
                        class="field h-11 !py-1.5 text-[12px] file:mr-3 file:rounded-md file:border-0 file:bg-surface-raised file:px-3 file:py-1.5 file:text-[12px] file:font-semibold file:text-text" />
                    <button @click="handleImportCsv" :disabled="!importFile || importLoading" type="button"
                        class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                        {{ importLoading ? $t("payroll.importingBtn") : $t("payroll.importBtn") }}
                    </button>
                    <button @click="downloadSalaryTemplate" type="button"
                        class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                        {{ $t("payroll.downloadTemplate") }}
                    </button>
                </div>
                <div v-if="importResult" class="text-xs">
                    <div v-if="importResult.error">
                        <div class="text-red-600">{{ importResult.error }}</div>
                        <ul v-if="importResult.errors && importResult.errors.length"
                            class="mt-1 text-red-600 space-y-0.5">
                            <li v-for="(e, i) in importResult.errors" :key="i">
                                Row {{ e.row }}<span v-if="e.rule"> · {{ e.rule }}</span>: {{ e.reason }}
                            </li>
                        </ul>
                    </div>
                    <div v-else>
                        <div>{{ $t("payroll.importSummary", {
                            employees: importResult.imported_employees, rules:
                            importResult.imported_rules }) }}</div>
                        <div v-if="importResult.missing && importResult.missing.length" class="text-red-600">
                            {{ $t("payroll.missingLabel") }} <span v-for="m in importResult.missing" :key="m.id">{{ m.id
                                }} {{ m.name }}<span v-if="m.reason"> ({{ m.reason }})</span>; </span>
                        </div>
                    </div>
                </div>
            </div>
            <SimpleModal :options="importConfirmModal.get()" @ok="confirmImportReplace" @cancel="cancelImportReplace"
                v-show="importConfirmModal.isShow()" />
            <div class="panel overflow-hidden flex flex-col">
                <div class="panel-head">
                    <span class="inline-flex items-center gap-2 min-w-0">
                        <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                        <span class="truncate">{{ $t("payroll.salaryTab") }}</span>
                        <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted hidden sm:inline">{{
                            $t("payroll.countRowsEmployees", {
                                shown: filteredSalaryTable.length, total:
                            employees.length }) }}</span>
                    </span>
                    <div class="flex items-center gap-1.5 shrink-0">
                        <button @click="loadEmployees(); loadSalaryTable()" type="button"
                            class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                            {{ $t("payroll.salaryReload") }}
                        </button>
                        <button @click="exportSalaryCsv" type="button"
                            class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                            {{ $t("payroll.exportBtn") }}
                        </button>
                        <button v-if="canEdit" @click="openSalaryModal" type="button"
                            class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                            {{ $t("payroll.salaryAssign") }}
                        </button>
                    </div>
                </div>
                <div class="table-shell hidden md:block max-h-[56vh]">
                    <table class="w-full min-w-[760px] border-collapse">
                        <thead class="sticky top-0 z-10">
                            <tr>
                                <th class="th w-[28%]">{{ $t("payroll.colEmployee") }}</th>
                                <th class="th w-[14%]">{{ $t("payroll.salaryType") }}</th>
                                <th class="th !text-right w-[16%]">{{ $t("payroll.salaryAmountPer", {
                                    unit:
                                    $t("payroll.salaryPerMonth") }) }}</th>
                                <th class="th !text-right w-[14%]">{{ $t("payroll.salaryHourly") }}</th>
                                <th class="th w-[20%]">{{ $t("payroll.salaryEffective") }}</th>
                                <th class="th !text-right w-[8%]"></th>
                            </tr>
                        </thead>
                        <tbody v-if="salaryTableLoading" class="bg-surface">
                            <tr v-for="n in 5" :key="n">
                                <td class="td">
                                    <div class="skeleton h-9"></div>
                                </td>
                                <td class="td">
                                    <div class="skeleton h-4"></div>
                                </td>
                                <td class="td">
                                    <div class="skeleton h-4"></div>
                                </td>
                                <td class="td">
                                    <div class="skeleton h-4"></div>
                                </td>
                                <td class="td">
                                    <div class="skeleton h-4"></div>
                                </td>
                            </tr>
                        </tbody>
                        <tbody v-else-if="filteredSalaryTable.length > 0" class="bg-surface">
                            <tr v-for="(row, idx) in filteredSalaryTable" :key="row.key"
                                class="even:bg-surface-raised/25">
                                <td class="td">
                                    <div class="flex items-center gap-3">
                                        <div class="w-9 h-9 rounded-xl flex items-center justify-center text-white mono text-[11px] font-extrabold shrink-0 shadow-sm ring-1 ring-border-strong"
                                            :style="{ background: salaryAvatar(row.employee) }">{{
                                            salaryInitials(row.employee) }}</div>
                                        <div class="min-w-0">
                                            <p class="text-[13px] font-bold text-text leading-none truncate">{{
                                                row.employee.first_name }} {{ row.employee.last_name || "" }}</p>
                                            <p class="mono text-[10px] tracking-wide text-text-muted mt-1 truncate">{{
                                                row.employee.department || "-" }}</p>
                                        </div>
                                    </div>
                                </td>
                                <td class="td">
                                    <span v-if="row.salary" class="chip chip-ok">{{ (row.salary.salary_type ||
                                        "monthly") === "daily" ? $t("payroll.salaryDailyType") :
                                        $t("payroll.salaryMonthlyType") }}</span>
                                    <span v-else class="chip chip-warn">{{ $t("payroll.salaryNone") }}</span>
                                </td>
                                <td class="td !text-right mono text-[13px] font-bold text-text">{{ row.salary ?
                                    `${salaryDisplayFor(row)}${(row.salary.salary_type || "monthly") === "daily" ? " " +
                                        $t("payroll.salaryPerDayShort") : " " + $t("payroll.salaryPerMonthShort")}` : "-" }}
                                </td>
                                <td class="td !text-right mono text-[11.5px] !text-text-muted">{{ row.salary ?
                                    fmtMoney(row.salary.hourly_rate) : "-" }}</td>
                                <td class="td mono text-[11.5px] !text-text-muted">{{ row.salary ?
                                    displayDate(row.salary.effective_date) : "-" }}</td>
                                <td class="td !text-right">
                                    <div class="flex justify-end gap-1.5">
                                        <button v-if="canEdit && row.salary" @click="openRulesEditor(row)" type="button"
                                            class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{
                                            $t("payroll.salaryRulesEditBtn") }}</button>
                                        <button v-if="canDelete && row.salary" @click="askDeleteSalary(row)"
                                            type="button"
                                            class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px]">{{
                                            $t("common.delete") }}</button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                        <tbody v-else>
                            <tr>
                                <td colspan="6">
                                    <div class="state">
                                        <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
                                        <p class="state-title">{{ $t("payroll.salaryEmpty") }}</p>
                                        <p class="state-hint">{{ $t("payroll.salaryEmptyHint") }}</p>
                                        <button v-if="canEdit" @click="handleAssignSalary"
                                            class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{
                                            $t("payroll.salaryAssign") }}</button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
                <div class="md:hidden p-3 space-y-3 max-h-[52vh] overflow-auto"
                    v-if="!salaryTableLoading && salaryTable.length > 0">
                    <div v-for="row in filteredSalaryTable" :key="`m-${row.key}`"
                        class="rounded-xl border border-border bg-surface p-3.5">
                        <div class="flex items-start justify-between gap-3">
                            <div class="flex items-center gap-2.5">
                                <div class="w-9 h-9 rounded-xl flex items-center justify-center text-white mono text-xs font-extrabold"
                                    :style="{ background: salaryAvatar(row.employee) }">{{ salaryInitials(row.employee)
                                    }}</div>
                                <div>
                                    <p class="text-sm font-bold text-text leading-none">{{ row.employee.first_name }} {{
                                        row.employee.last_name || "" }}</p>
                                    <p class="mono text-[10px] text-text-muted mt-1">{{ row.employee.department || "-"
                                        }}</p>
                                </div>
                            </div>
                            <span class="chip" :class="row.salary ? 'chip-ok' : 'chip-warn'">{{ row.salary ?
                                ((row.salary.salary_type || "monthly") === "daily" ? $t("payroll.salaryDailyType") :
                                $t("payroll.salaryMonthlyType")) : $t("payroll.salaryNone") }}</span>
                        </div>
                        <div class="mt-3 grid grid-cols-2 gap-2 mono text-xs">
                            <div class="rounded-lg bg-canvas border border-border px-2.5 py-2"><span
                                    class="text-text-muted">{{ $t("payroll.salaryAmountPer", {
                                        unit: row.salary &&
                                            (row.salary.salary_type || "monthly") === "daily" ? $t("payroll.salaryPerDayShort")
                                    : $t("payroll.salaryPerMonthShort") }) }}</span><span
                                    class="block font-bold text-text">{{ salaryDisplayFor(row) }}</span></div>
                            <div class="rounded-lg bg-canvas border border-border px-2.5 py-2"><span
                                    class="text-text-muted">{{ $t("payroll.salaryHourly") }}</span><span
                                    class="block font-bold text-text">{{ row.salary ? fmtMoney(row.salary.hourly_rate) :
                                    "-" }}</span></div>
                        </div>
                        <p class="mt-2 mono text-[11px] text-text-muted">{{ $t("payroll.salaryEffective") }}: {{
                            row.salary ? displayDate(row.salary.effective_date) : "-" }}</p>
                        <div v-if="row.salary" class="mt-2 flex justify-end gap-1.5">
                            <button v-if="canEdit" @click="openRulesEditor(row)" type="button"
                                class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{
                                    $t("payroll.salaryRulesEditBtn") }}</button>
                            <button v-if="canDelete" @click="askDeleteSalary(row)" type="button"
                                class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px]">{{ $t("common.delete")
                                }}</button>
                        </div>
                    </div>
                </div>
                <div v-if="!salaryTableLoading && salaryTable.length === 0" class="md:hidden">
                    <div class="state">
                        <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
                        <p class="state-title">{{ $t("payroll.salaryEmpty") }}</p>
                        <p class="state-hint">{{ $t("payroll.salaryEmptyHint") }}</p>
                        <button v-if="canEdit" @click="handleAssignSalary"
                            class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{ $t("payroll.salaryAssign")
                            }}</button>
                    </div>
                </div>
            </div>

            <SimpleModal :options="salaryModal.get()" wide @cancel="closeSalaryModal" v-show="salaryModal.isShow()">
                <div class="w-full space-y-4">
                    <!-- Rules-only edit: amounts and dates are locked, only rules can change. -->
                    <div v-if="rulesRow">
                        <div class="px-3 py-2 rounded-lg bg-surface-raised/60 border border-border text-xs text-text">
                            {{ rulesRow.employee.first_name }} {{ rulesRow.employee.last_name || "" }} - {{
                                displayDate(rulesRow.salary.effective_date) }}
                        </div>
                    </div>
                    <div v-if="!rulesRow">
                        <label class="label">
                            {{ $t("payroll.salaryEmployee") }}
                        </label>
                        <input v-model="salaryEmpQuery" @input="onSalaryEmpInput" @change="matchSalaryEmp"
                            @focus="onSalaryEmpFocus" @blur="salaryEmpOpen = false" @keydown="onSalaryEmpKey"
                            type="text" autocomplete="off" :placeholder="$t('payroll.salaryEmpSearch')" class="field" />
                        <ul v-if="salaryEmpOpen && salaryEmpFilter.length"
                            class="mt-1 max-h-44 overflow-auto rounded-xl border border-border bg-surface shadow-lg">
                            <li v-for="(e, i) in salaryEmpFilter" :key="e.id" @mousedown.prevent="pickSalaryEmp(e)"
                                :class="i === salaryEmpIdx ? 'bg-surface-sunken' : ''"
                                class="px-3 py-2.5 min-h-[44px] text-[13px] text-text cursor-pointer hover:bg-surface-raised">
                                {{ salaryEmpLabel(e) }}</li>
                        </ul>
                        <div v-if="salaryEmpOpen && !salaryEmpFilter.length"
                            class="mt-1 px-3 py-2 rounded-lg border border-border bg-surface-raised/50 text-xs text-text-muted">
                            {{ $t("payroll.salaryEmpEmpty") }}
                        </div>
                    </div>

                    <div v-if="!rulesRow" class="flex gap-2">
                        <button @click="salaryType = 'monthly'" type="button"
                            :class="salaryType === 'monthly' ? 'btn-primary' : 'btn-secondary'" class="btn flex-1">
                            {{ $t("payroll.salaryMonthlyType") }}
                        </button>
                        <button @click="salaryType = 'daily'" type="button"
                            :class="salaryType === 'daily' ? 'btn-primary' : 'btn-secondary'" class="btn flex-1">
                            {{ $t("payroll.salaryDailyType") }}
                        </button>
                    </div>

                    <div v-if="!rulesRow" class="grid grid-cols-2 gap-3">
                        <div v-if="salaryType === 'monthly'">
                            <label class="label">
                                {{ $t("payroll.salaryMonthly") }}
                            </label>
                            <input v-model="salaryMonthly" type="number" min="0" class="field" />
                        </div>
                        <div>
                            <label class="label">
                                {{ $t("payroll.salaryDaily") }}
                            </label>
                            <input v-model="salaryDaily" type="number" min="0" class="field" />
                        </div>
                        <div>
                            <label class="label">
                                {{ $t("payroll.salaryHourly") }}
                            </label>
                            <input v-model="salaryHourly" type="number" min="0" class="field" />
                        </div>
                        <div>
                            <label class="label">
                                {{ $t("payroll.salaryEffective") }}
                            </label>
                            <input v-model="salaryEffective" type="date" class="field w-full sm:!w-36 shrink-0" />
                        </div>
                    </div>

                    <div v-if="salaryError"
                        class="rounded-lg bg-danger-bg border border-red-200 px-3 py-2 text-[12.5px] text-danger">
                        {{ salaryError }}
                    </div>

                    <div class="pt-3 border-t border-border-strong">
                        <div class="flex items-center justify-between mb-2 gap-2">
                            <div class="text-xs font-semibold text-text">{{ $t("payroll.salaryRules") }}</div>
                            <div class="flex items-center gap-1.5">
                                <button
                                    @click="rulesHelpModal.setText(t('payroll.salaryRuleHelp'), ''); rulesHelpModal.showCancel()"
                                    type="button" :title="$t('payroll.salaryRuleHelp')"
                                    class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px] !px-0 w-9 shrink-0">?</button>
                                <select v-model="salaryCopyFrom" class="field max-w-[160px] text-xs">
                                    <option value="">{{ $t("payroll.salaryCopyFrom") }}</option>
                                    <option v-for="e in employees" :key="`cp-${e.id}`" :value="e.id">
                                        {{ salaryEmpLabel(e) }}
                                    </option>
                                </select>
                                <button @click="copySalaryRules" :disabled="!salaryCopyFrom || salaryCopyLoading"
                                    type="button" class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                                    {{ $t("payroll.salaryCopy") }}
                                </button>
                                <button @click="addSalaryRule" type="button"
                                    class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                                    {{ $t("payroll.salaryRuleAdd") }}
                                </button>
                            </div>
                        </div>
                        <SimpleModal :options="rulesHelpModal.get()" @cancel="rulesHelpModal.clear()"
                            v-show="rulesHelpModal.isShow()">
                            <div class="w-full max-w-full space-y-3 text-sm text-text">
                                <div class="text-xs font-semibold text-text-muted">{{ $t("payroll.salaryRuleVarsTitle")
                                    }}</div>
                                <!-- Per-variable table: name + meaning + scope (daily value vs period value). -->
                                <table
                                    class="w-full font-mono text-xs border border-border-strong rounded-lg overflow-hidden">
                                    <tbody class="divide-y divide-emerald-100 bg-surface-raised/60">
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">overtime</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarOt") }}</td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">absent</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarAbsent") }}</td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">present</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarPresent") }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">salary</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarSalary") }}</td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">monthly_rate</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarMonthly") }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">daily_rate</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarDaily") }}</td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">hourly_rate</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarHourly") }}</td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">late</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarLate") }}</td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">late_time</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarLateTime") }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">late_day</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarLateDay") }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">sick</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarSick") }}</td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">permission</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarPermission") }}
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">dow</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarDow") }}</td>
                                        </tr>
                                        <tr>
                                            <td class="px-2 py-1 font-bold whitespace-nowrap">hols</td>
                                            <td class="px-2 py-1 font-sans">{{ $t("payroll.salaryRuleVarHols") }}</td>
                                        </tr>
                                    </tbody>
                                </table>
                                <div class="text-xs font-semibold text-text-muted">{{
                                    $t("payroll.salaryRuleExamplesTitle") }}</div>
                                <!-- Ard chain: add first, then deducts. Later rules see salary mutated by earlier ones. -->
                                <ul class="font-mono text-xs space-y-1.5">
                                    <li class="bg-surface-raised border border-border rounded-lg px-3 py-1.5">{{
                                        $t("payroll.salaryRuleExOt") }}<br /><span class="text-text-muted">{{
                                            $t("payroll.salaryRuleExOtWhy") }}</span></li>
                                    <li class="bg-surface-raised border border-border rounded-lg px-3 py-1.5">{{
                                        $t("payroll.salaryRuleExBpjs") }}<br /><span class="text-text-muted">{{
                                            $t("payroll.salaryRuleExBpjsWhy") }}</span></li>
                                    <li class="bg-surface-raised border border-border rounded-lg px-3 py-1.5">{{
                                        $t("payroll.salaryRuleExPph") }}<br /><span class="text-text-muted">{{
                                            $t("payroll.salaryRuleExPphWhy") }}</span></li>
                                </ul>
                                <div class="text-xs text-text-muted">{{ $t("payroll.salaryRuleDailyNote") }}</div>
                                <div class="text-xs text-text-muted">{{ $t("payroll.salaryRuleSeqNote") }}</div>
                            </div>
                        </SimpleModal>
                        <div v-if="salaryRules.length === 0"
                            class="rounded-lg bg-surface-raised/60 border border-dashed border-border-strong px-3 py-2 text-[12px] text-text-muted">
                            {{ $t("payroll.salaryRulesEmpty") }}
                        </div>
                        <div v-else class="space-y-2">
                            <!-- Rule row stacks on mobile, 4 cols on sm+. Parameter gets widest share. -->
                            <div v-for="(rule, ridx) in salaryRules" :key="ridx" class="space-y-1.5">
                                <div
                                    class="grid grid-cols-1 sm:grid-cols-[minmax(120px,1fr)_110px_minmax(180px,2fr)_auto] gap-1.5 items-end">
                                    <div>
                                        <div class="label !mb-0.5">{{ $t("payroll.salaryRuleName") }}</div>
                                        <input v-model="rule.name" type="text"
                                            :placeholder="$t('payroll.salaryRuleName')" class="field text-xs" />
                                    </div>
                                    <div>
                                        <div class="label !mb-0.5">{{ $t("payroll.salaryRuleFunction") }}</div>
                                        <select v-model="rule.function" @change="onRuleFunction(rule)"
                                            class="field text-xs">
                                            <option value="Add">{{ $t("payroll.salaryRuleKindAdd") }}</option>
                                            <option value="Deduct">{{ $t("payroll.salaryRuleDeduct") }}</option>
                                        </select>
                                    </div>
                                    <div>
                                        <div class="label !mb-0.5">{{ $t("payroll.salaryRuleParameter") }}</div>
                                        <input v-model="rule.parameter" type="text"
                                            :placeholder="$t('payroll.salaryRuleParameterPh')"
                                            class="field mono text-xs" />
                                    </div>
                                    <div class="flex shrink-0 gap-1">
                                        <button @click="moveSalaryRule(ridx, -1)" :disabled="ridx === 0" type="button"
                                            :title="$t('payroll.salaryRuleMoveUp')"
                                            class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">^</button>
                                        <button @click="moveSalaryRule(ridx, 1)"
                                            :disabled="ridx === salaryRules.length - 1" type="button"
                                            :title="$t('payroll.salaryRuleMoveDown')"
                                            class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">v</button>
                                        <button @click="removeSalaryRule(ridx)" type="button"
                                            :title="$t('payroll.salaryRuleRemove')"
                                            class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px]">x</button>
                                    </div>
                                </div>
                                <div class="flex items-center justify-between">
                                    <label class="flex items-center gap-1.5 text-[11px] text-text cursor-pointer"
                                        :title="$t('payroll.salaryRuleDailyHint')">
                                        <input type="checkbox" v-model="rule.daily"
                                            class="accent-emerald-600 w-3.5 h-3.5" />
                                        {{ $t("payroll.salaryRuleDaily") }}
                                    </label>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-if="rulesError"
                        class="rounded-lg bg-danger-bg border border-red-200 px-3 py-2 text-[12.5px] text-danger">
                        {{ rulesError }}
                    </div>

                    <div class="flex gap-2">
                        <button @click="closeSalaryModal" type="button" class="btn btn-secondary flex-1">
                            {{ $t("common.cancel") }}
                        </button>
                        <button v-if="rulesRow" @click="handleSaveRules" :disabled="rulesSaving" type="button"
                            class="btn btn-primary flex-1">
                            {{ rulesSaving ? $t("payroll.salarySaving") : $t("payroll.salarySave") }}
                        </button>
                        <button v-else @click="handleAssignSalary" :disabled="salarySaving" type="button"
                            class="btn btn-primary flex-1">
                            {{ salarySaving ? $t("payroll.salarySaving") : $t("payroll.salarySave") }}
                        </button>
                    </div>
                </div>
            </SimpleModal>


        </div>

        <!-- Absence Tab -->
        <div v-show="tab === 'absence'" class="space-y-3">
            <!-- Absence add-condition card on top -->
            <div v-if="canEdit" class="panel p-3 sm:p-4 space-y-2">
                <div class="card-label">{{ $t("payroll.absAddTitle") }}</div>
                <div class="flex flex-col sm:flex-row gap-2">
                    <EmployeeSuggest ref="condEmpSuggest" :employees="employees"
                        :placeholder="$t('payroll.colEmployee')" input-class="field h-11 w-full sm:!w-64 !pr-9"
                        @pick="newCondition.employee_id = $event.id" @clear="newCondition.employee_id = ''" />
                    <input v-model="newCondition.work_date" type="date"
                        class="field w-full sm:!w-36 shrink-0 text-xs" />
                    <select v-model="newCondition.condition" class="field w-full sm:!w-44 shrink-0">
                        <option value="">{{ $t("payroll.colCondition") }}</option>
                        <option value="lupa_absen">{{ $t("payroll.conditionLupa_absen") }}</option>
                        <option value="cuti">{{ $t("payroll.conditionCuti") }}</option>
                        <option value="izin">{{ $t("payroll.conditionIzin") }}</option>
                        <option value="sakit">{{ $t("payroll.conditionSakit") }}</option>
                        <option value="absent">{{ $t("payroll.conditionAbsent") }}</option>
                    </select>
                    <input v-model="newCondition.reason" :placeholder="$t('payroll.colReason')" class="field" />
                    <button @click="addConditionRow" type="button" class="btn btn-primary">
                        {{ $t("payroll.addCondition") }}
                    </button>
                </div>
                <div v-if="dailyConditions.length > 0" class="bg-surface-raised rounded-lg px-3 py-2 text-xs text-text">
                    {{ $t("payroll.condQueued", { n: dailyConditions.length }) }} · {{ $t("payroll.condQueuedHint") }}
                </div>
            </div>
            <div v-if="absError"
                class="rounded-lg bg-danger-bg border border-red-200 px-3 py-2 text-[12.5px] text-danger max-w-2xl">
                {{ absError }}
            </div>
            <div v-else class="panel overflow-hidden flex flex-col">
                <div class="panel-head">
                    <span class="inline-flex items-center gap-2 min-w-0">
                        <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                        <span class="truncate">{{ $t("payroll.absenceTab") }}</span>
                        <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted hidden sm:inline">{{
                            $t("payroll.countShown", { shown: absences.length }) }}</span>
                    </span>
                    <div class="flex flex-wrap items-center justify-end gap-1.5">
                        <span class="card-label normal-case hidden sm:inline">{{ $t("common.filter") }}</span>
                        <EmployeeSuggest :employees="employees" :placeholder="$t('payroll.colEmployee')"
                            input-class="field h-9 !w-44 !pl-2 !pr-7 text-xs normal-case" @pick="pickAbsEmp"
                            @clear="clearAbsEmp" />
                        <select v-model="absStatus" @change="absPage = 0; loadAbsences()"
                            class="field h-9 !w-auto text-xs py-0 normal-case">
                            <option value="pending_verification">{{ $t("payroll.absPending") }}</option>
                            <option value="approved">{{ $t("payroll.absApprovedState") }}</option>
                            <option value="rejected">{{ $t("payroll.absRejectedState") }}</option>
                            <option value="all">{{ $t("payroll.absAll") }}</option>
                        </select>
                        <button @click="loadAbsences" type="button"
                            class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]"
                            :title="$t('common.refresh')">
                            <font-awesome :icon="faRotate" class="w-3.5 h-3.5" />
                        </button>
                    </div>
                </div>
                <div v-if="absences.length === 0" class="hidden md:block">
                    <div class="state">
                        <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
                        <p class="state-title">{{ absError || $t("payroll.absEmpty") }}</p>
                        <p class="state-hint">{{ $t("payroll.absEmptyHint") }}</p>
                    </div>
                </div>
                <div v-else class="table-shell hidden md:block max-h-[56vh]">
                    <table class="w-full min-w-[640px] border-collapse">
                        <thead class="sticky top-0 z-10">
                            <tr>
                                <th class="th">
                                    {{ $t("payroll.colEmployee") }}
                                </th>
                                <th class="th">
                                    {{ $t("payroll.colDate") }}
                                </th>
                                <th class="th">
                                    {{ $t("payroll.colCondition") }}
                                </th>
                                <th class="th">
                                    {{ $t("payroll.colStatus") }}
                                </th>
                                <th class="th !text-center">
                                    {{ $t("payroll.colActions") }}
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr v-for="a in absences" :key="a.id" class="hover:bg-surface-raised">
                                <td class="td">
                                    <span class="font-semibold text-text">{{ absenceEmployeeName(a.employee_id)
                                        }}</span>
                                    <span class="block text-[10px] font-mono text-text-muted">#{{ a.employee_id
                                        }}</span>
                                </td>
                                <td class="td text-[12px] !text-text-muted">
                                    {{ displayDate(a.absence_date) }}
                                </td>
                                <td class="td mono text-[11.5px]">{{ conditionLabel(a.absence_reason) }}</td>
                                <td class="td">
                                    {{ a.status === 'pending_verification' ? $t("payroll.absPending") : a.status ===
                                        'approved' ? $t("payroll.absApprovedState") : a.status === 'rejected' ?
                                    $t("payroll.absRejectedState") : a.status }}
                                </td>
                                <td class="td !text-center">
                                    <template v-if="a.status === 'pending_verification' && canEdit">
                                        <button @click="handleApproveAbsence(a.id)" type="button"
                                            class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">
                                            {{ $t("payroll.absApprove") }}
                                        </button>
                                        <button @click="askRejectAbsence(a)" type="button"
                                            class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px]">
                                            {{ $t("payroll.absReject") }}
                                        </button>
                                    </template>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- Mobile absence cards: same rows as the desktop table, stacked -->
            <div class="md:hidden space-y-2">
                <div v-if="absences.length === 0" class="panel">
                    <div class="state">
                        <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
                        <p class="state-title">{{ absError ? absError : $t("payroll.absEmpty") }}</p>
                        <p class="state-hint">{{ $t("payroll.absEmptyHint") }}</p>
                    </div>
                </div>
                <div v-for="a in absences" :key="`abs-m-${a.id}`"
                    class="rounded-xl border border-border bg-surface p-3.5 space-y-2">
                    <div class="flex items-start justify-between gap-2">
                        <div class="min-w-0">
                            <p class="text-sm font-bold text-text truncate">{{ absenceEmployeeName(a.employee_id) }}</p>
                            <p class="mono text-[11px] text-text-muted mt-0.5">#{{ a.employee_id }} · {{
                                displayDate(a.absence_date)
                                }} · {{ conditionLabel(a.absence_reason) }}</p>
                        </div>
                        <span class="chip shrink-0"
                            :class="a.status === 'approved' ? 'chip-ok' : a.status === 'rejected' ? 'chip-danger' : 'chip-warn'">
                            {{ a.status === 'pending_verification' ? $t("payroll.absPending") : a.status === 'approved'
                                ?
                                $t("payroll.absApprovedState") : a.status === 'rejected' ? $t("payroll.absRejectedState") :
                            a.status }}
                        </span>
                    </div>
                    <div v-if="a.status === 'pending_verification' && canEdit" class="flex gap-2 pt-1">
                        <button @click="handleApproveAbsence(a.id)" type="button"
                            class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px] flex-1">
                            {{ $t("payroll.absApprove") }}
                        </button>
                        <button @click="askRejectAbsence(a)" type="button"
                            class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px] flex-1">
                            {{ $t("payroll.absReject") }}
                        </button>
                    </div>
                </div>
            </div>
            <TablePager :page="absPage" :page-size="ABS_PAGE" :total="absTotal" @update:page="goAbsPage" />
        </div>
    </div>

    <SimpleModal :options="rejectModal.get()" @ok="confirmRejectAbsence" @cancel="cancelRejectAbsence"
        v-show="rejectModal.isShow()" />
</template>
