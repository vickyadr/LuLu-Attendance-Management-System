use crate::{
    receiver::r_cdata::*,
    utility::stor::{AppState, GenericResponse, KeyValResponse},
};
use actix_web::{get, post, web::{self, Bytes}, HttpResponse, Responder};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::{collections::HashMap};

#[get("/cdata")]
pub async fn get_cdata(data: web::Query<GetCData>) -> impl Responder {
    let mut text: String = "GET OPTION FROM: ".to_owned();
    println!("CDATA: {:?}", data.sn.as_deref().unwrap_or("No SN"));
    text.push_str(data.sn.as_deref().unwrap_or(""));
    text.push_str("\r\nStamp=0\r\n\
                        OpStamp=0\r\n\
                        ErrorDelay=60\r\n\
                        Delay=30\r\n\
                        TransTimes=00:00;14:05\r\n\
                        TransInterval=1\r\n\
                        TransFlag=1000000000\r\n\
                        TimeZone=7\r\n\
                        Realtime=1\r\n\
                        Encrypt=0");
    return HttpResponse::Ok().content_type("text/plain").body(text);
}

#[post("/cdata")]
pub async fn post_cdata(pool: web::Data<AppState>, get: web::Query<ReceiverCData>, body: Bytes) -> impl Responder {
    let mut data: HashMap<String, String> = HashMap::new();

    let data_body = String::from_utf8(body.to_vec()).unwrap();
    println!("POST CDATA: {:?}", data_body);
    
    if get.sn.is_none() {
        data.insert("sn".to_string(), "Serial Number cant be empty".to_string());
    }

    if get.table.is_none() {
        data.insert("table".to_string(), "Data table not found".to_string());
    }

    if get.stamp.is_none() && get.op_stamp.is_none() {
        data.insert("stamp".to_string(), "Timestamp cant be empty".to_string());
    }

    if data.len() > 0 {
        return HttpResponse::BadRequest().json(KeyValResponse::<String, String>::new(
            data,
            "Post Error".to_string(),
        ));
    }

    if get.table.as_ref().unwrap() == "OPERLOG" {

        return HttpResponse::Ok().content_type("text/plain")
            .body(data_operlog(pool, get, data_body).await);

    }else if get.table.as_ref().unwrap() == "ATTLOG" {


        return HttpResponse::Ok().content_type("text/plain")
            .body(data_attlog(pool, get, data_body).await);
    }

    let dt: DateTime<Utc> = Utc::now();

    return HttpResponse::InternalServerError().json(GenericResponse::<i8>::new(
        Vec::new(),
        dt.to_string(),//"Internal Error".to_string(),
    ));
}

async fn data_attlog(pool: web::Data<AppState>, get: web::Query<ReceiverCData>, data_body: String) -> String {

    let log_data = data_body.lines();
    println!("POST CDATA: {:?}", log_data);
    let mut log_recorded: i32 = 0;
    for logs in log_data {

        if logs.is_empty() {
            continue;
        }

        let log : Vec<&str> = logs.split("\t").collect();
        if log.len() < 6 {
            continue;
        }
        

        let enroll_type:i32;

        if log[2] == "1" {
            enroll_type = 1; // Fingerprint
        } else if log[3] == "1" {
            enroll_type = 2; // Password
        } else if log[4] == "1" {
            enroll_type = 3; // Card
        } else if log[5] == "1" {
            //enroll_type = 0;
            // Reseved
            continue; // Skip unsupported enroll types
        } else {
            continue; // Skip unsupported enroll types
        }

        let dt: NaiveDateTime = NaiveDateTime::parse_from_str(log[1], "%Y-%m-%d %H:%M:%S").unwrap_or(
            NaiveDateTime::default()
        );

        let query =
            sqlx::query(r#"INSERT INTO enrolls (enroll_device_sn, enroll_employee_id, enroll_type, enroll_time) VALUES ($1, $2, $3, $4)"#)
                .bind(get.sn.as_ref().unwrap().to_string())
                .bind(log[0].to_string())
                .bind(enroll_type)
                .bind(dt)
                .execute(&pool.db)
                .await
                .map_err(|e: sqlx::Error| e.to_string());
        
        if query.is_ok() {
            // Increment the log_recorded counter
            log_recorded += 1;
        }

        if let Err(_e) = query {
            //if e.contains("Duplicate entry") {
            //    return HttpResponse::BadRequest();
            //}
            println!("Error inserting log: {}", _e);
        }
    }

    return format!("OK {}", log_recorded);
}

async fn data_operlog(pool: web::Data<AppState>, get: web::Query<ReceiverCData>, data_body: String) -> String {

    let log_data = data_body.lines();
    let mut log_recorded: i32 = 0;
    for logs in log_data {

        if logs.is_empty() {
            continue;
        }

        let log : Vec<&str> = logs.split("\t").collect();
        if log.len() < 2 {
            continue;
        }

        let data_type = log[0].split_whitespace().next().unwrap_or("");

        match data_type {
            "OPLOG" => {
                // Process operation log
                let enroll_type:i32 = match log[0].split_whitespace().last().unwrap().parse::<i32>().unwrap_or(0) {
                    6 => 1, // Fingerprint
                    7 => 2, // Password
                    8 => 3, // Card
                    _ => continue, // Skip unsupported enroll types
                };

                let dt: NaiveDateTime = NaiveDateTime::parse_from_str(log[2], "%Y-%m-%d %H:%M:%S").unwrap_or(
                    NaiveDateTime::default()
                );

                let query =
                    sqlx::query(r#"INSERT INTO enrolls (enroll_device_sn, enroll_employee_id, enroll_type, enroll_time) VALUES ($1, $2, $3, $4)"#)
                        .bind(get.sn.as_ref().unwrap().to_string())
                        .bind(log[3].to_string())
                        .bind(enroll_type)
                        .bind(dt)
                        .execute(&pool.db)
                        .await
                        .map_err(|e: sqlx::Error| e.to_string());

                if query.is_ok() {
                    // Increment the log_recorded counter
                    log_recorded += 1;
                }

                if let Err(_e) = query {
                    //if e.contains("Duplicate entry") {
                    //    return HttpResponse::BadRequest();
                    //}
                    println!("Error inserting log: {}", _e);
                }
            },
            "USER" => {
                // Process user log
                let user_id:i32 = log[0].split("=").last().unwrap().parse::<i32>().unwrap_or(0);

                if user_id == 0 {
                    continue; // Skip if user ID is invalid
                }

                let dt: NaiveDateTime = NaiveDateTime::parse_from_str(log[2], "%Y-%m-%d %H:%M:%S").unwrap_or(
                    NaiveDateTime::default()
                );

                let check: bool = sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM employee WHERE employee_id=$1)"#)
                    .bind(user_id)
                    .fetch_one(&pool.db)
                    .await
                    .expect("Failed to check if employee exists");

                if check == false {
                    continue; // Skip if the user does not exist
                }

                let query =
                    sqlx::query(r#"UPDATE employee SET employee_first_name=$1, update_at=$2 WHERE employee_id=$3"#)
                        .bind(log[1].split("=").last().unwrap_or(""))
                        .bind(dt)
                        .bind(user_id)
                        .execute(&pool.db)
                        .await
                        .map_err(|e: sqlx::Error| e.to_string());

                if query.is_ok() {
                    // Increment the log_recorded counter
                    log_recorded += 1;
                }

                if let Err(_e) = query {
                    //if e.contains("Duplicate entry") {
                    //    return HttpResponse::BadRequest();
                    //}
                    println!("Error inserting log: {}", _e);
                }
            },
            "FP" => {
                // Process fingerprint log

                let user_id = log[0].split("=").last().unwrap().parse::<i32>().unwrap_or(0);

                if user_id == 0 {
                    continue; // Skip if user ID is invalid
                }

                let check: bool = sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM fingers WHERE finger_employee_id=$1 AND finger_id=$2)"#)
                    .bind(user_id)
                    .bind(log[1].split("=").last().unwrap_or(""))
                    .fetch_one(&pool.db)
                    .await
                    .expect("Failed to check if finger exists");

                if check {
                    let query =
                    sqlx::query(r#"UPDATE fingers SET finger_code=$1, update_at=$2 WHERE finger_employee_id=$3 AND finger_id=$4"#)
                        .bind(&log[4][4..])    
                        .bind(Utc::now())
                        .bind(user_id)
                        .bind(log[1].split("=").last().unwrap_or(""))
                        
                        .execute(&pool.db)
                        .await
                        .map_err(|e: sqlx::Error| e.to_string());
                    
                    if query.is_ok() {
                        // Increment the log_recorded counter
                        log_recorded += 1;
                    }

                    if let Err(_e) = query {
                        //if e.contains("Duplicate entry") {
                        //    return HttpResponse::BadRequest();
                        //}
                        println!("Error update log: {}", _e);
                    }
                }else{
                    let query =
                    sqlx::query(r#"INSERT INTO fingers (finger_employee_id, finger_code, finger_id) VALUES ($2, $3, $1)"#)
                        .bind(log[1].split("=").last().unwrap_or(""))
                        .bind(user_id)
                        .bind(&log[4][4..])
                        .execute(&pool.db)
                        .await
                        .map_err(|e: sqlx::Error| e.to_string());
                    
                    if query.is_ok() {
                        // Increment the log_recorded counter
                        log_recorded += 1;
                    }

                    if let Err(_e) = query {
                        //if e.contains("Duplicate entry") {
                        //    return HttpResponse::BadRequest();
                        //}
                        println!("Error inserting log: {}", _e);
                    }
                }

            },
            _ => {
                println!("Unknown data type: {}", data_type);
                continue; // Skip unknown types
            }
        }

    }

    if log_recorded == 0 {
        return format!("");
    }
    
    return format!("OK {}", log_recorded);
}
