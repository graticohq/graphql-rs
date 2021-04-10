use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription,  Schema};
use std::convert::Infallible;

use http;
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

//  let cookieSecret = var("COOKIE_SECRET").expect("COOKIE_SECRET is not set");

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
            (schema, request): (
        Schema<graphql::graphql::QueryRoot, EmptyMutation, EmptySubscription>,
        async_graphql::Request,
      )| {
        let jar = graphql::graphql::get_cookie_jar(cookie_header);
        async move {
          let request = request.data(jar.clone());
          let resp = schema.execute(request).await;
          let json = serde_json::to_string(&resp).unwrap();
          let reply = warp::reply::with_status(warp::reply::html(json), http::StatusCode::OK);
          let mut response = reply.into_response();

          let mut cookie_jar = jar.lock().await;
          let mut cookies = http::HeaderMap::new();
          for cookie in (*cookie_jar).iter_mut() {
            println!("setting cookie {}", cookie);
            cookies.append(
              http::header::SET_COOKIE,
              http::HeaderValue::from_str(cookie).unwrap(),
            );
          }
          response.headers_mut().extend(cookies);

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
