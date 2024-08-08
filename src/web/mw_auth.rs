use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::response::Response;
use axum::{async_trait, RequestPartsExt};
use axum::{extract::Request,middleware::Next};
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};
use crate::ctx::Ctx;
use crate::model::ModelController;
use crate::{Error, Result};
use crate::web::AUTH_TOKEN;

pub async fn mw_require_auth(
    ctx: Result<Ctx>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
    _mc: State<ModelController>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
    
    // Compute result Ctx
    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, _exp, _sign)) => {
            // Todo Token component validation
            Ok(Ctx::new(user_id))
        }
        Err(e) => Err(e)
    };

    // Remove cookie if something went wrong
    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie))
    {
        cookies.remove(Cookie::from(AUTH_TOKEN));
    }

    // store ctx into request 
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

// ctx extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()

    }
}

// Parse token of format user-[user-id].[expiration].signature
// returns (user_id, expiration, signature)
fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(
        r#"^user-(\d+)\.(.+)\.(.+)"#,
        &token,
    ).ok_or(Error::AuthFailTokenWontFormat)?;

    let user_id: u64 = user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWontFormat)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
}
