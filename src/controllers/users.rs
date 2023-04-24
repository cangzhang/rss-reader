use axum::{Json, Extension, http::StatusCode};
use sqlx::SqlitePool;

use crate::{models::users, errors};

pub async fn create_user(
  Json(form): Json<users::CreateUserForm>,
  Extension(db_pool): Extension<SqlitePool>,
) -> Result<(StatusCode, Json<users::User>), errors::CustomError> {
  let user = sqlx::query_as!(
      users::User,
      r#"INSERT INTO users (name, active) VALUES(?, 1)"#,
      form.name
  )
  .execute(&db_pool)
  .await
  .map_err(|_| errors::CustomError::InternalServerError)?;

  let user_id = user.last_insert_rowid();
  let user = sqlx::query_as!(
      users::User,
      r#"SELECT id, name, active FROM users where id = ?"#,
      user_id
  )
  .fetch_one(&db_pool)
  .await
  .map_err(|_| errors::CustomError::InternalServerError)?;

  Ok((StatusCode::CREATED, Json(user)))
}