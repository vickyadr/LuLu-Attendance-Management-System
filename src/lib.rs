pub mod models {
    pub mod m_command;
    pub mod m_user;
    pub mod m_login;
}

pub mod handler {
    pub mod adms {
        pub mod h_cdata;
        pub mod h_getrequest;
        pub mod h_devicecmd;
    }
    pub mod h_login;
}

pub mod receiver {
    pub mod r_cdata;
    pub mod r_login;
}

pub mod utility {
    pub mod db;
    pub mod router;
    pub mod stor;
    pub mod hash;
}