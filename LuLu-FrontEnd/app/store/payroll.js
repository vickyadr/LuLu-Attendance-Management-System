import { defineStore } from "pinia";
import { useAuthStore } from "~/store/auth";

export const usePayrollStore = defineStore("payroll", () => {
    const auth = useAuthStore();
    const config = useRuntimeConfig();
    const contents = ref([]);
    const payslipMeta = ref({ total: 0 });
    const absenceMeta = ref({ total: 0 });
    const generating = ref(false);
    const lastGenerate = ref(null);

    async function generate(period_start, period_end, daily_conditions = null, scope = {}) {
        generating.value = true;
        try {
            const payload = {
                period_start,
                period_end,
            };
            if (daily_conditions && daily_conditions.length > 0) {
                payload.daily_conditions = daily_conditions;
            }
            // Scope: exactly one of employee_id or department.
            if (scope.employee_id) payload.employee_id = scope.employee_id;
            else if (scope.department) payload.department = scope.department;

            const res = await apiFetch(`${config.public.apiBase}/payroll/payslips/generate`, {
                method: "POST",
                headers: auth.confHeaders(),
                body: payload,
            });

            if (res.code === 200 || res.code === 201) {
                lastGenerate.value = { period_start, period_end, count: res.data.length };
                return res;
            } else {
                throw new Error(res.message || "Generate failed");
            }
        } finally {
            generating.value = false;
        }
    }

    async function list(period_start = null, period_end = null, { limit = null, offset = 0 } = {}) {
        try {
            const params = new URLSearchParams();
            if (period_start) params.append("period_start", period_start);
            if (period_end) params.append("period_end", period_end);
            if (limit) { params.append("limit", String(limit)); params.append("offset", String(offset || 0)); }
            const qs = params.toString();
            const url = `${config.public.apiBase}/payroll/payslips${qs ? "?" + qs : ""}`;
            const res = await apiFetch(url, {
                method: "GET",
                headers: auth.confHeaders(),
            });
            if (res.code === 200) {
                const payload = Array.isArray(res.data) && Array.isArray(res.data[0]?.items)
                    ? res.data[0]
                    : { items: Array.isArray(res.data) ? res.data : [] };
                contents.value = payload.items || [];
                payslipMeta.value = { total: payload.total ?? contents.value.length };
                return contents.value;
            } else {
                throw new Error(res.message || "List failed");
            }
        } catch (e) {
            console.error("Payroll list error:", e);
            return [];
        }
    }

    async function listDailyAbsences(status = null, { limit = null, offset = 0, employeeId = null } = {}) {
        const params = new URLSearchParams();
        if (status) params.append("status", status);
        if (employeeId) params.append("employee_id", String(employeeId));
        if (limit) { params.append("limit", String(limit)); params.append("offset", String(offset || 0)); }
        const qs = params.toString();
        const url = `${config.public.apiBase}/payroll/daily-absences${qs ? "?" + qs : ""}`;
        const res = await apiFetch(url, {
            method: "GET",
            headers: auth.confHeaders(),
        });
        if (res.code === 200) {
            const payload = Array.isArray(res.data) && Array.isArray(res.data[0]?.items)
                ? res.data[0]
                : { items: Array.isArray(res.data) ? res.data : [] };
            absenceMeta.value = { total: payload.total ?? payload.items.length };
            return payload.items || [];
        }
        throw new Error(res.message || "Fetch failed");
    }

    async function lock(payslip_id) {
        try {
            const res = await apiFetch(`${config.public.apiBase}/payroll/payslips/${payslip_id}/lock`, {
                method: "POST",
                headers: auth.confHeaders(),
            });
            if (res.code === 200) {
                return res.data;
            } else {
                throw new Error(res.message || "Lock failed");
            }
        } catch (e) {
            console.error("Payroll lock error:", e);
            throw e;
        }
    }

    async function approveDailyAbsence(absence_id, notes = null) {
        try {
            const body = {};
            if (notes) body.notes = notes;

            const res = await apiFetch(`${config.public.apiBase}/payroll/daily-absences/${absence_id}/approve`, {
                method: "POST",
                headers: auth.confHeaders(),
                body,
            });
            if (res.code === 200) {
                return res.data;
            } else {
                throw new Error(res.message || "Approve failed");
            }
        } catch (e) {
            console.error("Payroll approve error:", e);
            throw e;
        }
    }

    async function rejectDailyAbsence(absence_id, notes = null) {
        try {
            const body = {};
            if (notes) body.notes = notes;

            const res = await apiFetch(`${config.public.apiBase}/payroll/daily-absences/${absence_id}/reject`, {
                method: "POST",
                headers: auth.confHeaders(),
                body,
            });
            if (res.code === 200) {
                return res.data;
            } else {
                throw new Error(res.message || "Reject failed");
            }
        } catch (e) {
            console.error("Payroll reject error:", e);
            throw e;
        }
    }

    // Delete one salary row. Versioning is append-only: add a new row with a newer date.
    async function removeSalary(id) {
        const res = await apiFetch(`${config.public.apiBase}/payroll/employee-salary/${id}`, {
            method: "DELETE",
            headers: auth.confHeaders(),
        });
        if (res.code === 200) return true;
        throw new Error(res.message || "Delete failed");
    }

    // Fetch all salary rows, including old versions (newest first from server).
    async function getAllSalaries() {
        const res = await apiFetch(`${config.public.apiBase}/payroll/employee-salaries`, {
            method: "GET",
            headers: auth.confHeaders(),
        });
        if (res.code === 200) return res.data || [];
        throw new Error(res.message || "Fetch failed");
    }

    // Update rules_json of one salary row. Amounts and dates stay locked.
    async function updateSalaryRules(id, rules_json) {
        const res = await apiFetch(`${config.public.apiBase}/payroll/employee-salary/${id}/rules`, {
            method: "PUT",
            headers: auth.confHeaders(),
            body: { rules_json },
        });
        if (res.code === 200) return res.data?.[0] || null;
        throw new Error(res.message || "Update failed");
    }

    async function assignSalary(payload) {
        const res = await apiFetch(`${config.public.apiBase}/payroll/employee-salary`, {
            method: "POST",
            headers: auth.confHeaders(),
            body: payload,
        });
        if (res.code !== 200) {
            // Keep the i18n key so the page can localize it (the demo mock never
            // passes through apiFetch's translate()).
            const error = new Error(res.message || "Assign failed");
            error.key = res.error || null;
            throw error;
        }
        return res.data;
    }

    async function getSalary(employee_id) {
        const res = await apiFetch(`${config.public.apiBase}/payroll/employee-salary/${employee_id}`, {
            method: "GET",
            headers: auth.confHeaders(),
        });
        if (res.code === 200) return res.data?.[0] || null;
        if (res.code === 404) return null;
        throw new Error(res.message || "Fetch failed");
    }

    async function getDetail(id) {
        const res = await apiFetch(`${config.public.apiBase}/payroll/payslips/${id}`, {
            method: "GET",
            headers: auth.confHeaders(),
        });
        if (res.code === 200) return res.data?.[0] || null;
        if (res.code === 404) return null;
        throw new Error(res.message || "Fetch failed");
    }

    // Delete a draft payslip. Locked rows are rejected by the server (409).
    async function removeDraft(id) {
        const res = await apiFetch(`${config.public.apiBase}/payroll/payslips/${id}`, {
            method: "DELETE",
            headers: auth.confHeaders(),
        });
        if (res.code === 200) return true;
        throw new Error(res.message || "Delete failed");
    }

    // Bulk delete draft payslips. Server skips locked rows, returns removed ids.
    async function removeDrafts(ids) {
        const res = await apiFetch(`${config.public.apiBase}/payroll/payslips/delete-drafts`, {
            method: "POST",
            headers: auth.confHeaders(),
            body: { ids },
        });
        if (res.code === 200) return res.data || [];
        throw new Error(res.message || "Delete failed");
    }

    return {
        contents,
        payslipMeta,
        absenceMeta,
        generating,
        lastGenerate,
        generate,
        list,
        listDailyAbsences,
        lock,
        approveDailyAbsence,
        rejectDailyAbsence,
        assignSalary,
        updateSalaryRules,
        removeSalary,
        getAllSalaries,
        getSalary,
        getDetail,
        removeDraft,
        removeDrafts,
    };
});
