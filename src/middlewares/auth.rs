// https://github.com/tokio-rs/axum/discussions/236#discussioncomment-1218395

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::http::Request;
use axum_core::response::Response;
use nanoid::nanoid;
use serde::Serialize;
use sqlx::SqlitePool;
use tower::{Layer, Service};
use tower_cookies::{Cookie, Cookies};

use crate::{controllers::users::COOKIE_USER_IDENT, errors::CustomError, models::sessions};

#[derive(Debug, Serialize, Default, Clone)]
pub struct Session {
    pub cookie_id: String,
    pub user_id: i64,
}

#[derive(Clone, Debug)]
pub struct SessionLayer {
    pub session: Arc<Session>,
}

impl SessionLayer {
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }
}

impl<S> Layer<S> for SessionLayer {
    type Service = SessionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionService {
            inner,
            session: self.session.clone(),
        }
    }
}

// middleware that
// 1. inserts a session into request extensions so handlers can access it
// 2. calls the inner service and awaits the response
// 3. prints the session so we can see if its changed
#[derive(Clone)]
pub struct SessionService<S> {
    inner: S,
    session: Arc<Session>,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SessionService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    #[allow(clippy::type_complexity)]
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let cookie_id = match req
                .extensions_mut()
                .get::<Cookies>()
                .ok_or(CustomError::InternalServerError)
            {
                Ok(cookies) => {
                    let mut id = cookies
                        .get(COOKIE_USER_IDENT)
                        .and_then(|c| c.value().parse().ok())
                        .unwrap_or(String::new());
                    if id.is_empty() {
                        id = nanoid!(8);
                        cookies.add(Cookie::new(COOKIE_USER_IDENT, id.clone()));
                    }

                    id
                }
                Err(_) => String::new(),
            };

            let user_id = match req
                .extensions_mut()
                .get::<SqlitePool>()
                .ok_or(CustomError::InternalServerError)
            {
                Ok(db_pool) => {
                    if cookie_id.is_empty() {
                        ()
                    }

                    let session_result = sqlx::query_as::<_, sessions::Session>(
                        "SELECT * FROM sessions WHERE cookie_id = $1",
                    )
                    .bind(&cookie_id)
                    .fetch_one(db_pool)
                    .await;

                    match session_result {
                        Ok(s) => s.user_id,
                        Err(_) => 0,
                    }
                }
                Err(_) => 0,
            };

            let session = Session { cookie_id, user_id };
            req.extensions_mut().insert(session.clone());
            let res = inner.call(req).await?;
            Ok(res)
        })
    }
}
