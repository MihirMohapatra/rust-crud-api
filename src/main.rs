use actix_web::{App, HttpResponse, HttpServer, Responder, ResponseError, http::StatusCode, web};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{PgConnection, prelude::*};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::{env, fmt};
use thiserror::Error;

mod schema;

use schema::users;

type DbPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
struct NewUser {
    name: String,
    email: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = users)]
struct UpdatedUser {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("database pool error")]
    Pool,
    #[error("database operation failed")]
    Database(#[from] diesel::result::Error),
    #[error("blocking task failed")]
    Blocking,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Database(diesel::result::Error::NotFound) => StatusCode::NOT_FOUND,
            ApiError::Pool | ApiError::Database(_) | ApiError::Blocking => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string(),
        })
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_owned());
    let bind_address = format!("{host}:{port}");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("failed to create database connection pool");

    println!("listening on http://{bind_address}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/users")
                    .route("", web::post().to(create_user))
                    .route("/{id}", web::get().to(get_user))
                    .route("/{id}", web::put().to(update_user))
                    .route("/{id}", web::delete().to(delete_user)),
            )
    })
    .bind(bind_address)?
    .run()
    .await
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

async fn create_user(
    pool: web::Data<DbPool>,
    payload: web::Json<NewUser>,
) -> Result<web::Json<User>, ApiError> {
    let new_user = payload.into_inner();
    run_db(pool, move |conn| {
        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
    })
    .await
    .map(web::Json)
}

async fn get_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<i32>,
) -> Result<web::Json<User>, ApiError> {
    let user_id = user_id.into_inner();
    run_db(pool, move |conn| {
        users::table
            .find(user_id)
            .select(User::as_select())
            .first(conn)
    })
    .await
    .map(web::Json)
}

async fn update_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<i32>,
    payload: web::Json<UpdatedUser>,
) -> Result<web::Json<User>, ApiError> {
    let user_id = user_id.into_inner();
    let updated_user = payload.into_inner();
    run_db(pool, move |conn| {
        diesel::update(users::table.find(user_id))
            .set(&updated_user)
            .returning(User::as_returning())
            .get_result(conn)
    })
    .await
    .map(web::Json)
}

async fn delete_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<i32>,
) -> Result<web::Json<User>, ApiError> {
    let user_id = user_id.into_inner();
    run_db(pool, move |conn| {
        diesel::delete(users::table.find(user_id))
            .returning(User::as_returning())
            .get_result(conn)
    })
    .await
    .map(web::Json)
}

async fn run_db<F, T>(pool: web::Data<DbPool>, query: F) -> Result<T, ApiError>
where
    F: FnOnce(&mut PgConnection) -> Result<T, diesel::result::Error> + Send + 'static,
    T: Send + 'static,
{
    web::block(move || {
        let mut conn = pool.get().map_err(|_| ApiError::Pool)?;
        query(&mut conn).map_err(ApiError::Database)
    })
    .await
    .map_err(|_| ApiError::Blocking)?
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.error)
    }
}
