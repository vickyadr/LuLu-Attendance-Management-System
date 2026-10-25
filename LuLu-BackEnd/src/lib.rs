pub mod models {
    pub mod m_command;
    pub mod m_device;
    pub mod m_employee;
    pub mod m_holiday;
    pub mod m_leave;
    pub mod m_login;
    pub mod m_report;
    pub mod m_schedule;
    pub mod m_settings;
    pub mod m_shift;
    pub mod m_transaction;
    pub mod m_user;
    pub mod payroll;
    pub mod payroll_employee_deductions;
}

pub mod handler {
    pub mod adms {
        pub mod h_cdata;
        pub mod h_devicecmd;
        pub mod h_getrequest;
    }
    pub mod h_device;
    pub mod h_zkpull;
    pub mod h_employee;
    pub mod h_holiday;
    pub mod h_field;
    pub mod h_leave;
    pub mod h_login;
    pub mod h_report;
    pub mod h_schedule;
    pub mod h_settings;
    pub mod h_shift;
    pub mod h_transaction;
    pub mod h_user;
    pub mod h_payroll;
    pub mod h_employee_deductions;
    pub mod payroll_api;
    pub mod payroll_calc_rules;
    pub mod payroll_expr;
    pub mod payroll_payslip;
}

pub mod receiver {
    pub mod r_cdata;
    pub mod r_csv_import;
    pub mod r_device;
    pub mod r_employee;
    pub mod r_login;
    pub mod r_schedule;
    pub mod r_shift;
    pub mod r_transaction;
    pub mod r_user;
}

pub mod service {
    pub mod live_scheduler;
    pub mod pull_scheduler;
}

pub mod utility {
    pub mod authorization;
    pub mod cors;
    pub mod db;
    pub mod device_sweeper;
    pub mod hash;
    pub mod helper;
    pub mod router;
    pub mod shutdown;
    pub mod stor;
    pub mod wshub;
    pub mod zkpull;
}
