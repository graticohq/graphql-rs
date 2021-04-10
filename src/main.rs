use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptyMutation, EmptySubscription, FieldResult, Object, Schema};
use async_graphql_warp::Response;
use std::convert::Infallible;
use std::sync::Arc;

use http;
use serde_json::Value;
use warp::{http::Response as HttpResponse, Filter, Reply};

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

#[tokio::main]
async fn main() {
  dotenv().ok();
  let pg_pool: PgPool = db_connection().await.expect("Database connection failed.");
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
  .data(pg_pool)
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
        let jar = graphql::graphql::get_cookie_jar(cookie_header);
        async move {
          let j = jar.clone();
          let request = request.data(j);
          let resp = schema.execute(request).await;
          let mut cookie_jar = jar.lock().await;
          let j = serde_json::to_string(&resp).unwrap();

          let reply = warp::reply::html(j);
          let reply = warp::reply::with_status(reply, http::StatusCode::OK);
          let mut cookies = http::HeaderMap::new();
          for key in (*cookie_jar).iter_mut() {
            println!("{}", key);
            cookies.append(
              http::header::SET_COOKIE,
              http::HeaderValue::from_str(key).unwrap(),
            );
          }
          let mut response = reply.into_response();
          let headers = response.headers_mut();
          headers.extend(cookies);
          Ok::<_, Infallible>(response)
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
