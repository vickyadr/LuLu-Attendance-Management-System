pub mod models {
    pub mod m_command;
    pub mod m_user;
    pub mod m_login;
    pub mod m_transaction;
    pub mod m_device;
    pub mod m_shift;
    pub mod m_schedule;
    pub mod m_report;
    pub mod m_employee;
}

pub mod handler {
    pub mod adms {
        pub mod h_cdata;
        pub mod h_getrequest;
        pub mod h_devicecmd;
    }
    pub mod h_login;
    pub mod h_transaction;
    pub mod h_device;
    pub mod h_shift;
    pub mod h_schedule;
    pub mod h_report;
    pub mod h_employee;
}

pub mod receiver {
    pub mod r_cdata;
    pub mod r_login;
    pub mod r_transaction;
    pub mod r_shift;
    pub mod r_device;
    pub mod r_schedule;
    pub mod r_employee;
}

pub mod utility {
    pub mod db;
    pub mod router;
    pub mod stor;
    pub mod hash;
    pub mod authorization;
    pub mod helper;
}