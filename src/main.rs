use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{
    Context,  EmptyMutation, EmptySubscription, Object, Schema, FieldResult
};
use async_graphql_warp::{ Response};
use std::convert::Infallible;
use warp::{http::Response as HttpResponse, Filter};

use sqlx::postgres::{PgPool, PgPoolOptions};
use dotenv::dotenv;
use std::env::var;

mod posts;

struct QueryRoot;
#[Object]
impl QueryRoot {
    async fn posts<'a>(&self, ctx: &Context<'_>) -> FieldResult<i64> {
        let pool = ctx.data_unchecked::<PgPool>();
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api.projects")
            .fetch_one(pool)
            .await?;
            Ok(count)
        }
    
}
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub async fn db_connection() -> Result<PgPool> {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url).await?;
    Ok(pool)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pg_pool: PgPool = db_connection().await.expect("Database connection failed.");
    let port = var("PORT").expect("PORT is not set").parse::<u16>().ok().expect("PORT should be an integer");

    println!("Booting graphQL Server on port {}", port);

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(pg_pool)
        .finish();

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let graphql_post = async_graphql_warp::graphql(schema.clone()).and_then(
        |(schema,  request): (
            Schema<QueryRoot, EmptyMutation, EmptySubscription>,
            async_graphql::Request,
        )| async move {
            let resp = schema.execute(request).await;
            Ok::<_, Infallible>(Response::from(resp))
        },
    );

    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(
                GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
            ))
    });

    let routes = hello
    .or(graphql_playground)
    .or(graphql_post);

    println!("Playground: http://localhost:{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
