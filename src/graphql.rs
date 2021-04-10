pub mod graphql {

  use async_graphql::{Context, FieldResult, Object};
  use cookie::{Cookie, ParseError};
  use std::sync::{Arc};
  use tokio::sync::Mutex;

  use sqlx::postgres::PgPool;

  pub type CookieJar = Arc<Mutex<Vec<String>>>;

  pub fn get_cookiejar_mutex(items: Vec<String>) -> CookieJar {
    Arc::new(Mutex::new(items))
  }

  pub fn from_cookie_header(header: String) -> std::result::Result<CookieJar, ParseError> {
    let mut vec: Vec<String> = Vec::new();
    let list = header.split("; ");
    for cookiestr in list {
      let cookie = Cookie::parse(cookiestr.to_owned())?;
      vec.push(cookie.name().to_string())
    }
    let  jar_mutex = get_cookiejar_mutex(vec);
    Ok(jar_mutex)
  }

  pub fn get_cookie_jar(cookie_header: Option<String>) -> CookieJar {
    let jar = match cookie_header {
      Some(header) => match from_cookie_header(header.to_owned()) {
        Ok(jar) => jar,
        Err(_) => get_cookiejar_mutex(Vec::new()),
      },
      None => get_cookiejar_mutex(Vec::new())
    };
    return jar;
  }


  pub struct QueryRoot;
  #[Object]
  impl QueryRoot {
    async fn posts<'a>(&self, ctx: &Context<'_>) -> FieldResult<i64> {
      let jar = ctx.data_unchecked::<CookieJar>();
      let pg_pool = ctx.data_unchecked::<PgPool>();
      let mut cookie_jar = jar.lock().await;
      let cookie = Cookie::new("name", "value");
      (*cookie_jar).push(cookie.to_string());
      let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api.projects")
        .fetch_one(pg_pool)
        .await?;
      Ok(count)
    }
  }
}
