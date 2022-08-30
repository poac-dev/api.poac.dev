mod create_user;
mod find_user;
mod get_access_token;
mod get_email;
mod get_user_meta;

use create_user::create_user;
use find_user::find_user;
use get_access_token::get_access_token;
use get_email::get_email;
use get_user_meta::get_user_meta;

use crate::user::models::User;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::http::header;
use actix_web::{get, web, HttpResponse, Result};
use diesel::prelude::*;
use poac_api_utils::{DbError, DbPool};
use serde::Deserialize;

#[derive(Deserialize)]
struct Query {
    code: String,
}

#[get("/auth/callback")]
async fn auth_callback(
    pool: web::Data<DbPool>,
    web::Query(query): web::Query<Query>,
) -> Result<HttpResponse> {
    let access_token = get_access_token(query.code).await?;
    let user_meta = get_user_meta(&access_token).await?;
    let maybe_user = find_user(pool.clone(), user_meta.clone()).await?;
    let user = match maybe_user {
        None => {
            // Create a new user so that there was not the same user.
            let email = get_email(&access_token).await?;
            create_user(pool.clone(), user_meta, email).await?
        }
        Some(user) => {
            // This is NOT a new user
            if user.status != "active" {
                log::warn!("A disabled user tried to log in: {:?}", user);
                return Err(ErrorUnauthorized("You are not authorized."));
            }

            // Update if user info is stale (user_meta must be up-to-date)
            if user_meta.name != user.name || user_meta.avatar_url != user.avatar_url {
                web::block(move || -> Result<User, DbError> {
                    use crate::schema::users::dsl::{avatar_url, id, name, user_name, users};

                    let mut conn = pool.get()?;
                    let user = diesel::update(users)
                        .filter(user_name.eq(user_meta.user_name))
                        .set((
                            name.eq(&user_meta.name),
                            avatar_url.eq(&user_meta.avatar_url),
                        ))
                        .returning((id, name, user_name, avatar_url))
                        .get_result::<User>(&mut conn)?;

                    Ok(user)
                })
                .await?
                .map_err(ErrorInternalServerError)?
            } else {
                User::from(user)
            }
        }
    };

    let base64_user = base64::encode(serde_json::to_string(&user)?.as_bytes());
    let redirect_uri = format!(
        "https://poac.pm/api/auth?access_token={}&user_metadata={}",
        access_token, base64_user
    );
    Ok(HttpResponse::TemporaryRedirect()
        .append_header((header::LOCATION, redirect_uri))
        .finish())
}

pub(crate) fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(auth_callback);
}
