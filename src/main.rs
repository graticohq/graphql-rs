use std::convert::Infallible;
use dotenv::dotenv;
use std::env::var;

use sqlx::postgres::{PgPool, PgPoolOptions};
use warp::{http::Response as HttpResponse, Filter, Rejection};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

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

pub fn get_graphql_playground_route() -> impl Filter<Extract = (std::result::Result<http::Response<String>, http::Error>,), Error = Rejection> + Clone {
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

  //  let cookieSecret = var("COOKIE_SECRET").expect("COOKIE_SECRET is not set");

  let schema = Schema::build(graphql::QueryRoot, EmptyMutation, EmptySubscription)
    .data(pg_pool)
    .finish();

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
        let jar = graphql::cookies::get_cookie_jar(cookie_header);
        async move {
          let graphql_request = graphql_request.data(jar.clone());
          let graphql_response = schema.execute(graphql_request).await;
          let graphql_json = serde_json::to_string(&graphql_response).unwrap();
          let response =
            graphql::cookies::respond_with_jar(&jar, graphql_json).await;
          Ok::<_, Infallible>(response)
        }
      },
    );


  let routes = hello.or(graphql_playground).or(graphql_post);
  println!("Playground - http://localhost:{} ", port);
  warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
