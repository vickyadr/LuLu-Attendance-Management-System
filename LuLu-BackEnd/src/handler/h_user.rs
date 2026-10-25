use crate::{
    models::m_user::*,
    receiver::r_user::*,
    utility::{
        authorization::{Action, check_action_access, get_session_user, get_user_permissions},
        hash::hash_password,
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};
use actix_web::{
    Responder, get, post,
    web::{self, ReqData},
};
use std::collections::HashMap;

/// Gate the user-management area. Access is decided purely by the 'users' page
/// permission: holding `can_view` there is what makes someone an administrator of
/// accounts, so there is no separate admin flag or numeric level.
macro_rules! require_users_action {
    ($pool:expr, $bearer:expr, $action:expr) => {
        if let Ok(resp) =
            check_action_access($pool.db.clone(), $bearer.clone(), "users", $action).await
        {
            return resp;
        }
    };
}

#[get("/user/list")]
pub async fn user_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    require_users_action!(pool, bearer, Action::View);

    match sqlx::query_as::<_, User>(
        r##"SELECT user_id, user_nickname, user_fname, user_lname, user_password FROM users ORDER BY user_id ASC"##,
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(users) => {
            let mut result = Vec::new();
            for user in users {
                let perms = get_user_permissions(pool.db.clone(), &user.user_nickname).await;
                result.push(UserWithPermissions {
                    user_id: user.user_id,
                    user_nickname: user.user_nickname,
                    user_fname: user.user_fname,
                    user_lname: user.user_lname,
                    permissions: perms,
                });
            }
            GenericResponse::<UserWithPermissions>::ok(result, "ok".to_string()).into_response()
        }
        Err(e) => {
            log::error!("Error: {}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[post("/user/add")]
pub async fn user_add(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverUserAdd>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    require_users_action!(pool, bearer, Action::Add);

    let mut data: HashMap<&str, String> = HashMap::new();

    match body.nickname.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("nickname", "Too short".to_string());
            }
        }
        None => {
            data.insert("nickname", "Field required".to_string());
        }
    }

    match body.password.clone() {
        Some(o) => {
            if o.len().lt(&8) {
                data.insert("password", "Too short".to_string());
            }
        }
        None => {
            data.insert("password", "Field required".to_string());
        }
    }

    match body.fname.clone() {
        Some(o) => {
            if o.len().lt(&2) {
                data.insert("fname", "Too short".to_string());
            }
        }
        None => {
            data.insert("fname", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There was an error while adding user".to_string(),
            400,
        )
        .into_response();
    }

    let password_encode = hash_password(&body.password.clone().unwrap_or_default()).await;

    // A new account gets no permission rows, which denies every page and every
    // action. Access is granted afterwards, explicitly, through
    // POST /user/permissions - creating a user must not silently hand over rights.
    match sqlx::query_scalar::<_, String>(
        r##"INSERT INTO users (user_nickname, user_password, user_fname, user_lname) VALUES ($1, $2, $3, $4) RETURNING user_nickname"##,
    )
    .bind(body.nickname.clone())
    .bind(password_encode)
    .bind(body.fname.clone())
    .bind(body.lname.clone())
    .fetch_one(&pool.db)
    .await
    {
        Ok(user_nickname) => {
            GenericResponse::<String>::ok(vec![user_nickname], "User added".to_string())
                .into_response()
        }
        Err(e) => {
            log::error!("Error: {}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[post("/user/edit")]
pub async fn user_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverUserEdit>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    require_users_action!(pool, bearer, Action::Edit);

    let mut data: HashMap<&str, String> = HashMap::new();

    let user_nickname = match body.user_nickname.clone() {
        Some(id) => id,
        None => {
            return NoDataResponse::new("user_nickname required".to_string(), 400).into_response();
        }
    };

    // Confirm the row exists before building the UPDATE, so a typo in the nickname
    // reports "User not found" instead of silently affecting zero rows.
    let existing: Option<String> =
        sqlx::query_scalar(r##"SELECT user_nickname FROM users WHERE user_nickname = $1"##)
            .bind(&user_nickname)
            .fetch_optional(&pool.db)
            .await
            .ok()
            .flatten();

    if existing.is_none() {
        return NoDataResponse::new("User not found".to_string(), 404).into_response();
    }

    // Column names are hardcoded below; only values are bound. Every bound value
    // is a String now that the numeric level column is gone, so a single bind list
    // is enough and cannot drift into a Postgres type mismatch.
    let mut updates: Vec<String> = Vec::new();
    let mut binds: Vec<String> = Vec::new();
    let mut param_idx = 1;

    if let Some(nickname) = body.nickname.clone() {
        if nickname.len().lt(&3) {
            data.insert("nickname", "Too short".to_string());
        } else {
            updates.push(format!("user_nickname = ${}", param_idx));
            binds.push(nickname);
            param_idx += 1;
        }
    }

    let mut password_changed = false;
    if let Some(password) = body.password.clone() {
        if password.len().lt(&8) {
            data.insert("password", "Too short".to_string());
        } else {
            let encoded = hash_password(&password).await;
            updates.push(format!("user_password = ${}", param_idx));
            binds.push(encoded);
            param_idx += 1;
            password_changed = true;
        }
    }

    if let Some(fname) = body.fname.clone() {
        if fname.len().lt(&2) {
            data.insert("fname", "Too short".to_string());
        } else {
            updates.push(format!("user_fname = ${}", param_idx));
            binds.push(fname);
            param_idx += 1;
        }
    }

    if let Some(lname) = body.lname.clone() {
        updates.push(format!("user_lname = ${}", param_idx));
        binds.push(lname);
        param_idx += 1;
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There was an error while editing user".to_string(),
            400,
        )
        .into_response();
    }

    if updates.is_empty() {
        return NoDataResponse::new("No fields to update".to_string(), 400).into_response();
    }

    let sql = format!(
        "UPDATE users SET {} WHERE user_nickname = ${}",
        updates.join(", "),
        param_idx
    );

    // user_nickname is the PK and the permission FK carries ON UPDATE CASCADE, so
    // a rename moves the permission rows across with the user.
    let mut query = sqlx::query(sqlx::AssertSqlSafe(sql));
    for value in binds {
        query = query.bind(value);
    }
    query = query.bind(&user_nickname);

    // A reset is also the containment step for a stolen token: the old
    // session must die in the same transaction, like the field reset does.
    let final_nickname = body
        .nickname
        .clone()
        .filter(|nickname| nickname.len() >= 3)
        .unwrap_or(user_nickname);
    let mut tx = match pool.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Error: {}", e);
            return NoDataResponse::new("Internal server error".to_string(), 500).into_response();
        }
    };

    match query.execute(&mut *tx).await {
        Ok(_) => {
            if password_changed {
                let user_id = sqlx::query_scalar::<_, i32>(
                    "SELECT user_id FROM users WHERE user_nickname = $1",
                )
                .bind(&final_nickname)
                .fetch_one(&mut *tx)
                .await;
                let user_id = match user_id {
                    Ok(user_id) => user_id,
                    Err(e) => {
                        log::error!("Error: {}", e);
                        return NoDataResponse::new("Internal server error".to_string(), 500)
                            .into_response();
                    }
                };
                if let Err(e) = sqlx::query("DELETE FROM sessions WHERE session_user_id = $1")
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await
                {
                    log::error!("Error: {}", e);
                    return NoDataResponse::new("Internal server error".to_string(), 500)
                        .into_response();
                }
            }
            match tx.commit().await {
                Ok(_) => NoDataResponse::new("OK".to_string(), 200).into_response(),
                Err(e) => {
                    log::error!("Error: {}", e);
                    NoDataResponse::new("Internal server error".to_string(), 500).into_response()
                }
            }
        }
        Err(e) => {
            log::error!("Error: {}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[post("/user/delete")]
pub async fn user_delete(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverUserDelete>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let current_user_nickname = match get_session_user(pool.db.clone(), bearer.clone()).await {
        Some(user) => user,
        None => {
            return NoDataResponse::new("Session token invalid".to_string(), 401).into_response();
        }
    };

    require_users_action!(pool, bearer, Action::Delete);

    let user_nickname = match body.user_nickname.clone() {
        Some(id) => id,
        None => {
            return NoDataResponse::new("user_nickname required".to_string(), 400).into_response();
        }
    };

    // Deleting your own row would revoke your own permissions mid-request and
    // leave nobody able to undo it.
    if current_user_nickname == user_nickname {
        return NoDataResponse::new("Cannot delete yourself".to_string(), 400).into_response();
    }

    // The FK cascades, so the target's permission rows go with the user.
    match sqlx::query(r##"DELETE FROM users WHERE user_nickname = $1"##)
        .bind(&user_nickname)
        .execute(&pool.db)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                NoDataResponse::new("User not found".to_string(), 404).into_response()
            } else {
                NoDataResponse::new("OK".to_string(), 200).into_response()
            }
        }
        Err(e) => {
            log::error!("Error: {}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[post("/user/permissions")]
pub async fn user_permissions_update(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverUserPermissionsUpdate>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let current_user_nickname = match get_session_user(pool.db.clone(), bearer.clone()).await {
        Some(user) => user,
        None => {
            return NoDataResponse::new("Session token invalid".to_string(), 401).into_response();
        }
    };

    require_users_action!(pool, bearer, Action::Edit);

    let user_nickname = match body.user_nickname.clone() {
        Some(id) => id,
        None => {
            return NoDataResponse::new("user_nickname required".to_string(), 400).into_response();
        }
    };

    let user_exists: Option<String> =
        sqlx::query_scalar(r##"SELECT user_nickname FROM users WHERE user_nickname = $1"##)
            .bind(&user_nickname)
            .fetch_optional(&pool.db)
            .await
            .ok()
            .flatten();

    if user_exists.is_none() {
        return NoDataResponse::new("User not found".to_string(), 404).into_response();
    }

    // Refuse to strip your own access to user management. With no admin level and
    // no superuser escape hatch, revoking your own 'users' permission is a
    // one-way door that can lock every account out of the app.
    //
    // Write access must be preserved too, not just view. THIS endpoint is gated on
    // can_edit, so dropping your own can_edit makes the save succeed and then
    // locks you out of every future permission change - including the one that
    // would restore it. can_add and can_delete are safe to drop: /user/add and
    // /user/delete check them independently, and view still lets you manage
    // others, so no capability becomes unreachable.
    if current_user_nickname == user_nickname {
        let mut keeps_view = false;
        let mut keeps_edit = false;
        for p in body.permissions.iter() {
            if p.page_key == "users" {
                keeps_view = p.can_view;
                keeps_edit = p.can_edit;
            }
        }
        if !keeps_view {
            return NoDataResponse::new(
                "You cannot remove your own user management access".to_string(),
                400,
            )
            .into_response();
        }
        if !keeps_edit {
            return NoDataResponse::new(
                "You cannot remove your own edit access to user management".to_string(),
                400,
            )
            .into_response();
        }
    }

    // Four independent flags per page, written as one upsert so a page the admin
    // left out of the payload keeps its previous value instead of resetting.
    for perm in &body.permissions {
        match sqlx::query(
            r##"INSERT INTO user_page_permissions
                   (user_nickname, page_key, can_view, can_add, can_edit, can_delete)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (user_nickname, page_key) DO UPDATE SET
                   can_view = $3, can_add = $4, can_edit = $5, can_delete = $6,
                   updated_at = CURRENT_TIMESTAMP"##,
        )
        .bind(&user_nickname)
        .bind(&perm.page_key)
        .bind(perm.can_view)
        .bind(perm.can_add)
        .bind(perm.can_edit)
        .bind(perm.can_delete)
        .execute(&pool.db)
        .await
        {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error updating permission: {}", e);
                return NoDataResponse::new("Internal server error".to_string(), 500)
                    .into_response();
            }
        }
    }

    NoDataResponse::new("OK".to_string(), 200).into_response()
}

#[get("/user/permissions")]
pub async fn user_permissions_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    // Deliberately not gated on the 'users' page: every signed-in user needs their
    // OWN flags to render the sidebar and guard the routes they can reach.
    // Reading someone else's permissions still goes through /user/list, which
    // requires 'users' + can_view.
    match get_session_user(pool.db.clone(), bearer).await {
        Some(user_nickname) => {
            match sqlx::query_as::<_, UserPermission>(
                r##"SELECT page_key, can_view, can_add, can_edit, can_delete
                    FROM user_page_permissions WHERE user_nickname = $1 ORDER BY page_key ASC"##,
            )
            .bind(&user_nickname)
            .fetch_all(&pool.db)
            .await
            {
                Ok(perms) => {
                    GenericResponse::<UserPermission>::ok(perms, "OK".to_string()).into_response()
                }
                Err(e) => {
                    log::error!("Error: {}", e);
                    NoDataResponse::new("Internal server error".to_string(), 500).into_response()
                }
            }
        }
        None => NoDataResponse::new("Session token invalid".to_string(), 401).into_response(),
    }
}
