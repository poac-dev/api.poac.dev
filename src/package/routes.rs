use crate::package::actions;
use crate::utils::{DbPool, Response};
use actix_web::error::ErrorInternalServerError;
use actix_web::{get, post, web, HttpResponse, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParam {
    filter: Option<String>,
}

#[get("/v1/packages")]
async fn get_all(
    pool: web::Data<DbPool>,
    web::Query(query): web::Query<QueryParam>,
) -> Result<HttpResponse> {
    let packages = web::block(move || {
        let mut conn = pool.get()?;
        actions::get_all(&mut conn, query.filter)
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    Ok(Response::ok(packages))
}

#[derive(Deserialize)]
struct SearchBody {
    query: String,
    per_page: Option<i64>,
}

#[post("/v1/search")]
async fn search(
    pool: web::Data<DbPool>,
    web::Json(body): web::Json<SearchBody>,
) -> Result<HttpResponse> {
    let packages = web::block(move || {
        let mut conn = pool.get()?;
        actions::search(&mut conn, &body.query, body.per_page)
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    Ok(Response::ok(packages))
}

#[derive(Deserialize, Clone)]
struct NameVerBody {
    name: String,
    version: String,
}

#[post("/v1/repoinfo")]
async fn repo_info(pool: web::Data<DbPool>, body: web::Json<NameVerBody>) -> Result<HttpResponse> {
    let body_ = body.clone();

    let packages = web::block(move || {
        let mut conn = pool.get()?;
        actions::repo_info(&mut conn, &body_.name, &body_.version)
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    let body_ = body.into_inner();
    Ok(Response::maybe_ok(
        packages,
        format!(
            "No package found where name = `{}` & version = `{}`",
            body_.name, body_.version
        ),
    ))
}

async fn versions_impl(pool: web::Data<DbPool>, name: String) -> Result<HttpResponse> {
    let packages = web::block(move || {
        let mut conn = pool.get()?;
        actions::versions(&mut conn, &name)
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    Ok(Response::ok(packages))
}

#[get("/v1/packages/{org}/{name}/versions")]
async fn versions(
    pool: web::Data<DbPool>,
    full_name: web::Path<(String, String)>,
) -> Result<HttpResponse> {
    let (org, name) = full_name.into_inner();
    versions_impl(pool, format!("{}/{}", org, name)).await
}

#[get("/v1/packages/{name}/versions")]
async fn versions_official(
    pool: web::Data<DbPool>,
    name: web::Path<String>,
) -> Result<HttpResponse> {
    versions_impl(pool, name.into_inner()).await
}

#[post("/v1/deps")]
async fn deps(pool: web::Data<DbPool>, body: web::Json<NameVerBody>) -> Result<HttpResponse> {
    let body_ = body.clone();

    let packages = web::block(move || {
        let mut conn = pool.get()?;
        actions::deps(&mut conn, &body_.name, &body_.version)
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    let body_ = body.into_inner();
    Ok(Response::maybe_ok(
        packages,
        format!(
            "No package found where name = `{}` & version = `{}`",
            body_.name, body_.version
        ),
    ))
}

pub(crate) fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all);
    cfg.service(search);
    cfg.service(repo_info);
    cfg.service(versions);
    cfg.service(versions_official);
    cfg.service(deps);
}
