// https://github.com/tokio-rs/axum/discussions/236#discussioncomment-1218395

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use axum::http::{self, Request};
use axum_core::response::Response;
use nanoid::nanoid;
use sqlx::SqlitePool;
use tower::{Layer, Service};
use tower_cookies::{Cookie, Cookies};

use crate::{controllers::users::COOKIE_USER_IDENT, errors::CustomError, models::sessions};

#[derive(Debug, Default, Clone)]
pub struct Session {
    pub cookie_id: String,
    pub user_id: i64,
}

impl Session {
    pub fn new(cookie_id: String, user_id: i64) -> Self {
        Self { cookie_id, user_id }
    }
}

#[derive(Clone, Debug)]
pub struct SessionService<S> {
    inner: S,
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

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
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

            let session = Session::new(cookie_id, user_id);

            // Add the application state to the request's extension
            req.extensions_mut().insert(session.clone());

            let res = inner.call(req).await?;
            Ok(res)
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct SessionManagerLayer;

impl<S> Layer<S> for SessionManagerLayer {
    type Service = SessionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionService { inner }
    }
}
