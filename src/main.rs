use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptyMutation, EmptySubscription, FieldResult, Object, Schema};
use async_graphql_warp::Response;
use std::convert::Infallible;
use std::fmt;
use std::sync::Arc;
use warp::{http::Response as HttpResponse, Filter};

use cookie::{Cookie, CookieJar, Key, ParseError};

use dotenv::dotenv;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env::var;

mod graphql;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub async fn db_connection() -> Result<PgPool> {
  let database_url = var("DATABASE_URL").expect("DATABASE_URL is not set");
  let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
  Ok(pool)
}

pub fn from_cookie_header<'a>(header: String) -> std::result::Result<CookieJar, ParseError> {
  let mut ret = CookieJar::default();
  let list = header.split("; ");
  for cookiestr in list {
    ret.add_original(Cookie::parse(cookiestr.to_owned())?);
  }
  Ok(ret)
}

pub fn get_cookie_jar(cookie_header: Option<String>) -> CookieJar {
  let jar = match cookie_header {
    Some(header) => {
      match from_cookie_header(header.to_owned()) {
        Ok(jar) => jar,
        Err(_) => CookieJar::default(),
      }
    },
    None => CookieJar::default(),
  };
  return jar;
}

pub fn get_cookie_header(jar: CookieJar) -> String {
  return jar
    .delta()
    .map(|s| s.to_string())
    .collect::<Vec<String>>()
    .join(";");
}

pub fn get_context(db_pool: &PgPool, jar: &CookieJar) {
  let ctx = graphql::graphql::GraphQLContext {
    db_pool: db_pool,
    cookie_jar: jar
  };
  ctx;
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  let pg_pool:  PgPool = db_connection().await.expect("Database connection failed.");
  let port = var("PORT")
    .expect("PORT is not set")
    .parse::<u16>()
    .ok()
    .expect("PORT should be an integer");

  println!("Booting graphQL Server on port {}", port);

  let cookieSecret = var("COOKIE_SECRET").expect("COOKIE_SECRET is not set");

  let schema = Schema::build(
    graphql::graphql::QueryRoot,
    EmptyMutation,
    EmptySubscription,
  )
  .finish();

  let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
  let graphql_post = warp::header::optional::<String>("cookie")
    .and(async_graphql_warp::graphql(schema))
    .and_then(
      move |cookie_header: Option<String>,
            (schema, mut request): (
        Schema<graphql::graphql::QueryRoot, EmptyMutation, EmptySubscription>,
        async_graphql::Request,
      )| {
        let jar = get_cookie_jar(cookie_header);
        let ctx  =  get_context(&pg_pool, &jar);
      async move {
            request = request.data(ctx);
          let resp = schema.execute(request).await;
          let header = get_cookie_header(jar);
          println!("{}", header);
          Ok::<_, Infallible>(warp::reply::with_header(
            Response::from(resp),
            warp::http::header::SET_COOKIE,
            header,
          ))
        }
      },
    );

  let graphql_playground = warp::path::end().and(warp::get()).map(|| {
    HttpResponse::builder()
      .header("content-type", "text/html")
      .body(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
      ))
  });

  let routes = hello.or(graphql_playground).or(graphql_post);
  println!("Playground - http://localhost:{} ", port);
  warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
