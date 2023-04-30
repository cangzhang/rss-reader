use axum::{http::StatusCode, Extension, Json};
use bcrypt::{hash, DEFAULT_COST};
use nanoid::nanoid;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};

use crate::{errors, middlewares, models::sessions, models::users};

pub const COOKIE_USER_IDENT: &str = "user_identity";

pub async fn create_user(
    Extension(db_pool): Extension<SqlitePool>,
    _cookies: Cookies,
    Json(form): Json<users::CreateUserForm>,
) -> Result<(StatusCode, Json<users::User>), errors::CustomError> {
    let password_hash = hash(form.password, DEFAULT_COST).unwrap();
    let user = sqlx::query_as!(
        users::User,
        r#"INSERT INTO users (name, password_hash, active) VALUES(?1, ?2, 1)"#,
        form.name,
        password_hash,
    )
    .execute(&db_pool)
    .await
    .map_err(|_| errors::CustomError::InternalServerError)?;

    let user_id = user.last_insert_rowid();
    let user = sqlx::query_as!(
        users::User,
        r#"SELECT id, name, active, password_hash FROM users where id = ?"#,
        user_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(|_| errors::CustomError::InternalServerError)?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    Extension(db_pool): Extension<SqlitePool>,
    cookies: Cookies,
    Json(form): Json<users::LoginForm>,
) -> Result<(StatusCode, Json<users::User>), errors::CustomError> {
    let cookie_id = cookies
        .get(COOKIE_USER_IDENT)
        .and_then(|c| c.value().parse().ok())
        .unwrap_or(String::new());
    if cookie_id.is_empty() {
        let id = nanoid!(8);
        cookies.add(Cookie::new(COOKIE_USER_IDENT, id));
    }

    let user = sqlx::query_as!(
        users::User,
        r#"SELECT id, name, active, password_hash FROM users WHERE name = ?"#,
        form.name,
    )
    .fetch_one(&db_pool)
    .await
    .map_err(|_| errors::CustomError::InvalidCredentials)?;

    let valid = bcrypt::verify(&form.password, &user.password_hash);
    if let Ok(valid) = valid {
        if valid {
            let session_result = sqlx::query_as::<_, sessions::Session>(
                "SELECT * FROM sessions WHERE user_id = $1 AND cookie_id = $2",
            )
            .bind(user.id)
            .bind(&cookie_id)
            .fetch_one(&db_pool)
            .await;

            println!("{:?}", session_result);

            match session_result {
                Ok(s) => {
                    let _r = sqlx::query(
                        "UPDATE sessions SET last_active = datetime('now') WHERE id = $1",
                    )
                    .bind(s.id)
                    .execute(&db_pool)
                    .await
                    .map_err(|_| errors::CustomError::InternalServerError)?;
                }
                Err(_) => {
                    let _r = sqlx::query_as!(
                        sessions::Session,
                        r#"INSERT OR REPLACE INTO sessions (cookie_id, user_id) VALUES(?1, ?2)"#,
                        cookie_id,
                        user.id
                    )
                    .execute(&db_pool)
                    .await
                    .map_err(|_| errors::CustomError::InternalServerError)?;
                }
            }

            return Ok((StatusCode::OK, Json(user)));
        }
    }

    return Err(errors::CustomError::BadRequest);
}

pub async fn list(
    Extension(session): Extension<middlewares::auth::Session>,
) -> Result<(StatusCode, Json<Value>), errors::CustomError> {
    let s = session.inner.lock().unwrap();

    return Ok((
        StatusCode::OK,
        Json(json!({
            "cookie_id": *s.cookie_id,
            "user_id": s.user_id,
        })),
    ));
}
