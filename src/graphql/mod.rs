use async_graphql::{Context, FieldResult, Object};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use sqlx::postgres::PgPool;

mod cookies;

pub async fn execute_graphql_request_with_cookies(
  schema: Schema<QueryRoot, EmptyMutation, EmptySubscription>,
  graphql_request: async_graphql::Request,
  cookie_header: Option<String>,
) -> http::Response<warp::hyper::Body> {
  let jar = cookies::get_cookie_jar(cookie_header);
  let graphql_request = graphql_request.data(jar.clone());
  let graphql_response = schema.execute(graphql_request).await;
  let graphql_json = serde_json::to_string(&graphql_response).unwrap();
  let response = cookies::respond_with_jar(&jar, graphql_json).await;
  return response;
}

pub fn get_schema(pg_pool: PgPool, cookie_secret: String) -> Schema<QueryRoot, EmptyMutation, EmptySubscription> {
  let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
    .data(pg_pool)
    .data(cookie_secret)
    .finish();
  return schema;
}

pub struct QueryRoot;
#[Object]
impl QueryRoot {
  async fn posts<'a>(&self, ctx: &Context<'_>) -> FieldResult<i64> {
    let jar = ctx.data_unchecked::<cookies::CookieJar>();
    let pg_pool = ctx.data_unchecked::<PgPool>();
    let cookie_secret = ctx.data_unchecked::<String>();
    println!("{}", cookie_secret);
    let cv = cookies::get_cookie(jar, "n2".to_owned()).await;
    match cv {
      Some(v) => println!("found v {}", v),
      None => println!("non"),
    }
    cookies::set_cookie(&jar, "n2".to_string(), "vvv".to_string()).await;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api.projects")
      .fetch_one(pg_pool)
      .await?;
    Ok(count)
  }
}
