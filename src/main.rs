use dotenv::dotenv;
use std::convert::Infallible;
use std::env::var;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use sqlx::postgres::{PgPool, PgPoolOptions};
use warp::{http::Response as HttpResponse, Filter, Rejection};

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

pub fn get_hello_route() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
  warp::path!("hello" / String).map(|name| format!("Hello, {}!", name))
}

pub fn get_graphql_playground_route() -> impl Filter<
  Extract = (std::result::Result<http::Response<String>, http::Error>,),
  Error = Rejection,
> + Clone {
  warp::path::end().and(warp::get()).map(|| {
    HttpResponse::builder()
      .header("content-type", "text/html")
      .body(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
      ))
  })
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

  let cookie_secret = var("COOKIE_SECRET").expect("COOKIE_SECRET is not set");

  let schema = graphql::get_schema(pg_pool, cookie_secret);

  let hello = get_hello_route();
  let graphql_playground = get_graphql_playground_route();

  let graphql_post = warp::header::optional::<String>("cookie")
    .and(async_graphql_warp::graphql(schema))
    .and_then(
      move |cookie_header: Option<String>,
            (schema, graphql_request): (
        Schema<graphql::QueryRoot, EmptyMutation, EmptySubscription>,
        async_graphql::Request,
      )| {
        async move {
          let response =
            graphql::execute_graphql_request_with_cookies(schema, graphql_request, cookie_header)
              .await;
          Ok::<_, Infallible>(response)
        }
      },
    );

  let routes = hello.or(graphql_playground).or(graphql_post);
  println!("Playground - http://localhost:{} ", port);
  warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
