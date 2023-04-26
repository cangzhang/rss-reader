use axum::{http::StatusCode, Extension, Json};
use bcrypt::{hash, DEFAULT_COST};
use sqlx::SqlitePool;
use tower_cookies::{Cookie, Cookies};

use crate::{errors, models::users};

const COOKIE_NAME: &str = "visited";

pub async fn create_user(
    Extension(db_pool): Extension<SqlitePool>,
    cookies: Cookies,
    Json(form): Json<users::CreateUserForm>,
) -> Result<(StatusCode, Json<users::User>), errors::CustomError> {
    let visited = cookies
        .get(COOKIE_NAME)
        .and_then(|c| c.value().parse().ok())
        .unwrap_or(0);
    if visited > 10 {
        cookies.remove(Cookie::new(COOKIE_NAME, ""));
        println!("Reset visited count to {}", visited);
    } else {
        cookies.add(Cookie::new(COOKIE_NAME, (visited + 1).to_string()));
        println!("You've been here {} times before", visited);
    }

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
    Json(form): Json<users::LoginForm>,
) -> Result<(StatusCode, Json<users::User>), errors::CustomError> {
    let user = sqlx::query_as!(
        users::User,
        r#"SELECT id, name, active, password_hash FROM users where name = ?"#,
        form.name,
    )
    .fetch_one(&db_pool)
    .await
    .map_err(|_| errors::CustomError::InternalServerError)?;

    let valid = bcrypt::verify(&form.password, &user.password_hash);

    if let Ok(valid) = valid {
        if valid {
            return Ok((StatusCode::OK, Json(user)));
        }
    }

    return Err(errors::CustomError::BadRequest);
}
