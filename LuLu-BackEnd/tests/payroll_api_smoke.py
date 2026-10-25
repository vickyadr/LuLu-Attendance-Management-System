import json
import os
import shlex
import subprocess
import urllib.error
import urllib.request


def request(base_url, method, path, token=None, body=None, expected=None):
    data = None if body is None else json.dumps(body).encode()
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(base_url + path, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=10) as response:
            status, payload = response.status, response.read()
    except urllib.error.HTTPError as error:
        status, payload = error.code, error.read()
    result = json.loads(payload) if payload else {}
    assert status == expected, (method, path, status, result)
    return result


def psql(database, sql):
    command = shlex.split(os.environ.get("PAYROLL_PSQL_CMD", "psql"))
    return subprocess.check_output(
        command + ["-X", "-qAt", "-v", "ON_ERROR_STOP=1", "-d", database, "-c", sql],
        text=True,
    ).strip()


def main():
    database = os.environ.get("PAYROLL_TEST_DATABASE", "")
    if not database.startswith("lulu_payroll_"):
        raise SystemExit("PAYROLL_TEST_DATABASE must name a throwaway lulu_payroll_* database")
    base_url = os.environ.get("PAYROLL_BASE_URL", "http://127.0.0.1:4344").rstrip("/")
    nickname = os.environ.get("PAYROLL_TEST_USER", "demo")
    password = os.environ.get("PAYROLL_TEST_PASSWORD")
    if not password:
        raise SystemExit("Set PAYROLL_TEST_PASSWORD for the throwaway API user")
    assert psql(database, "SELECT current_database()") == database

    restricted_nickname = "payroll_denied"
    psql(database, f"DELETE FROM users WHERE user_nickname='{restricted_nickname}'")
    psql(
        database,
        "INSERT INTO users (user_password, user_nickname, user_fname) "
        f"SELECT user_password, '{restricted_nickname}', 'Payroll' FROM users WHERE user_nickname='{nickname}'",
    )
    psql(
        database,
        "INSERT INTO user_page_permissions (user_nickname, page_key, can_view) "
        f"VALUES ('{restricted_nickname}', 'settings', true)",
    )
    employee_id = int(
        psql(
            database,
            "INSERT INTO employee (employee_fname, employee_lname, employee_department, employee_status, employee_schedule_id) "
            "VALUES ('E2E', 'Payroll', 'Test', 0, 1) RETURNING employee_id",
        )
    )
    try:
        restricted_login = request(
            base_url,
            "POST",
            "/api/login",
            body={"nickname": restricted_nickname, "password": password},
            expected=200,
        )
        request(
            base_url,
            "GET",
            "/api/payroll/payslips",
            restricted_login["data"][0]["token"],
            expected=403,
        )
        login = request(
            base_url,
            "POST",
            "/api/login",
            body={"nickname": nickname, "password": password},
            expected=200,
        )
        token = login["data"][0]["token"]

        request(
            base_url,
            "POST",
            "/api/payroll/payslips/generate",
            token,
            {"period_start": "2026-10-02", "period_end": "2026-10-31"},
            400,
        )
        salary_request = {
            "employee_id": employee_id,
            "monthly_rate": 2_000_000,
            "basic_salary": 2_000_000,
            "effective_date": "2026-10-01",
        }
        salary_created = request(
            base_url, "POST", "/api/payroll/employee-salary", token, salary_request, 200
        )
        assert type(salary_created["data"][0]["employee_id"]) is int
        salary_read = request(
            base_url,
            "GET",
            f"/api/payroll/employee-salary/{employee_id}",
            token,
            expected=200,
        )
        assert salary_read["data"][0]["employee_id"] == employee_id
        psql(
            database,
            "INSERT INTO attendance_daily (employee_id, work_date, total_hours, days_credited, full_days, remainder_hours, ot_hours, daily_pay, ot_pay, holiday_premium, total_day_pay) VALUES "
            f"({employee_id}, DATE '2026-10-01', 8, 1, 1, 0, 0, 100000, 0, 0, 100000), "
            f"({employee_id}, DATE '2026-10-02', 8, 1, 1, 0, 0, 100000, 0, 0, 100000)",
        )
        period = {"period_start": "2026-10-01", "period_end": "2026-10-31"}
        created = request(
            base_url, "POST", "/api/payroll/payslips/generate", token, period, 201
        )
        assert len(created["data"]) == 1
        payslip = created["data"][0]
        payslip_id = payslip["id"]
        assert payslip["employee_id"] == employee_id
        assert payslip["regular_days_pay"] == 200_000
        assert payslip["gross_salary"] == 200_000
        assert payslip["total_deductions"] == 18_000
        assert payslip["net_salary"] == 182_000
        assert len(payslip["lines"]) == 2
        listed = request(base_url, "GET", "/api/payroll/payslips", token, expected=200)
        assert len(listed["data"]) == 1
        fetched = request(
            base_url, "GET", f"/api/payroll/payslips/{payslip_id}", token, expected=200
        )
        assert fetched["data"][0]["id"] == payslip_id
        assert len(fetched["data"][0]["lines"]) == 2
        request(base_url, "POST", "/api/payroll/payslips/generate", token, period, 409)
        locked = request(
            base_url,
            "POST",
            f"/api/payroll/payslips/{payslip_id}/lock",
            token,
            expected=200,
        )
        assert locked["data"][0]["status"] == "locked"
        request(
            base_url,
            "POST",
            f"/api/payroll/payslips/{payslip_id}/lock",
            token,
            expected=409,
        )
        locked_read = request(
            base_url, "GET", f"/api/payroll/payslips/{payslip_id}", token, expected=200
        )
        assert locked_read["data"][0]["locked_by_user_nickname"] == nickname
        counts = psql(
            database,
            f"SELECT (SELECT count(*) FROM salaries WHERE employee_id={employee_id}), "
            f"(SELECT count(*) FROM attendance_daily WHERE employee_id={employee_id}), "
            f"(SELECT count(*) FROM payslips WHERE employee_id={employee_id}), "
            f"(SELECT count(*) FROM payslip_lines l JOIN payslips p ON p.id=l.payslip_id "
            f"WHERE p.employee_id={employee_id} AND p.status='locked')",
        )
        assert counts == "1|2|1|2", counts
        print(
            json.dumps(
                {
                    "salary_insert_read": 200,
                    "period_validation": 400,
                    "generate": 201,
                    "list": 200,
                    "get": 200,
                    "duplicate_period": 409,
                    "lock": 200,
                    "repeat_lock": 409,
                    "persisted_counts": counts,
                    "employee_id": employee_id,
                    "payslip_id": payslip_id,
                },
                indent=2,
            )
        )
    finally:
        psql(database, f"DELETE FROM employee WHERE employee_id={employee_id}")
        remaining = psql(
            database,
            f"SELECT (SELECT count(*) FROM salaries WHERE employee_id={employee_id}), "
            f"(SELECT count(*) FROM attendance_daily WHERE employee_id={employee_id}), "
            f"(SELECT count(*) FROM payslips WHERE employee_id={employee_id}), "
            f"(SELECT count(*) FROM payslip_lines l JOIN payslips p ON p.id=l.payslip_id "
            f"WHERE p.employee_id={employee_id})",
        )
        assert remaining == "0|0|0|0", remaining
        psql(database, "DELETE FROM users WHERE user_nickname='payroll_denied'")
        print("throwaway_rows_after_cleanup=" + remaining)


if __name__ == "__main__":
    main()
