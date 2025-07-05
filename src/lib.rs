pub mod models {
    pub mod m_content;
    pub mod m_command;
}

pub mod handler {
    pub mod adms {
        pub mod h_cdata;
        pub mod h_getrequest;
        pub mod h_devicecmd;
    }
    pub mod h_list;
}

pub mod receiver {
    pub mod r_list;
    pub mod r_cdata;
}

pub mod utility {
    pub mod db;
    pub mod router;
    pub mod stor;
}
