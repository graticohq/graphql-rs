pub mod cookies {
  use cookie::{Cookie, ParseError};
  use std::sync::Arc;
  use time;
  use tokio::sync::Mutex;
  use warp::{Reply};

  pub type CookieJar = Arc<Mutex<Vec<String>>>;

  pub fn get_cookiejar_mutex(items: Vec<String>) -> CookieJar {
    Arc::new(Mutex::new(items))
  }

  pub fn from_cookie_header(header: String) -> std::result::Result<CookieJar, ParseError> {
    let mut vec: Vec<String> = Vec::new();
    let list = header.split("; ");
    for cookiestr in list {
      println!("got cookir {}", cookiestr);
      let cookie = Cookie::parse(cookiestr.to_owned());
      match cookie {
        Ok(_) => vec.push(cookiestr.to_string()),
        Err(_) => (),
      }
    }
    let jar_mutex = get_cookiejar_mutex(vec);
    Ok(jar_mutex)
  }

  pub fn get_cookie_jar(cookie_header: Option<String>) -> CookieJar {
    let jar = match cookie_header {
      Some(header) => match from_cookie_header(header.to_owned()) {
        Ok(jar) => jar,
        Err(_) => get_cookiejar_mutex(Vec::new()),
      },
      None => get_cookiejar_mutex(Vec::new()),
    };
    return jar;
  }

  pub async fn get_cookie(jar: &CookieJar, name: String) -> Option<String> {
    let  cookie_jar = jar.lock().await;
    let mut list = (*cookie_jar).iter().map(
      |x| Cookie::parse(x.to_owned())
    );
    let index = list.find(|c| {
      return match c {
        Ok(v) => v.name() == name,
        Err(_) => false
      }
    });
    return match index {
      Some(v) => match v {
        Ok(c) => Some(c.value().to_string()),
        Err(_) => None
      },
      None => None
    }
  }

  pub async fn set_cookie(jar: &CookieJar, name: String, value: String) {
    let mut cookie_jar = jar.lock().await;
    let cookie = Cookie::build(name, value)
      .path("/")
      .secure(false)
      .http_only(true)
      .expires(time::OffsetDateTime::now_utc() + time::Duration::days(365))
      .finish();
    (*cookie_jar).push(cookie.to_string());
  }

  pub async fn respond_with_jar(jar: &CookieJar, string: String) -> http::Response<warp::hyper::Body> {
    let reply = warp::reply::with_status(warp::reply::html(string), http::StatusCode::OK);
    let mut response = reply.into_response();
    let mut cookie_jar = jar.lock().await;
    let mut cookies_map = http::HeaderMap::new();
    for cookie_str in (*cookie_jar).iter_mut() {
      let cookie = Cookie::parse(cookie_str.to_owned());
      match cookie {
        Ok(cookie) => {
          match cookie.expires() {
            Some(_) => {
              println!("setting cookie {}", cookie_str);
              cookies_map.append(
                http::header::SET_COOKIE,
                http::HeaderValue::from_str(cookie_str).unwrap(),
              );      
            },
            None => ()
          }
        },
        Err(_) => (),
      }

    }
    response.headers_mut().extend(cookies_map);
    return response;
  }
}

use async_graphql::{Context, FieldResult, Object};

use sqlx::postgres::PgPool;

pub struct QueryRoot;
#[Object]
impl QueryRoot {
  async fn posts<'a>(&self, ctx: &Context<'_>) -> FieldResult<i64> {
    let jar = ctx.data_unchecked::<cookies::CookieJar>();
    let pg_pool = ctx.data_unchecked::<PgPool>();
    let cv = cookies::get_cookie(jar, "n2".to_owned()).await;
    match cv {
      Some(v) => println!("found v {}", v),
      None => println!("non")
    }
    cookies::set_cookie(&jar, "n2".to_string(), "vvv".to_string()).await;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api.projects")
      .fetch_one(pg_pool)
      .await?;
    Ok(count)
  }
}
