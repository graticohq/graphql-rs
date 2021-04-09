
pub mod graphql {
    
    use async_graphql::{Context, FieldResult, Object};
    use cookie::{Cookie, CookieJar, Key, ParseError};
    use http::header::{AsHeaderName, HeaderMap, IntoHeaderName};

    
    use sqlx::postgres::{PgPool};

    #[derive(Debug, Clone)]
    pub struct GraphQLContext {
         pub cookie_jar:  CookieJar
    }
    impl<'b> GraphQLContext {
        pub fn new( cookie_jar:  CookieJar) -> Self {
            GraphQLContext { 
                cookie_jar
             }
        }
        } 

        pub fn get_cookie_header(jar: CookieJar) -> Vec<String> {
            return jar
              .delta()
              .map(|s| s.to_string())
              .collect::<Vec<String>>()
          }
          

    pub struct QueryRoot;
    #[Object]
    impl QueryRoot {
        async fn posts<'a>(&self, ctx: &Context<'_>) -> FieldResult<i64> {
            let c = ctx.data_unchecked::<GraphQLContext>();
            let pg_pool = ctx.data_unchecked::<PgPool>();
            let mut jar = c.cookie_jar.to_owned();
            jar.add(Cookie::new("second", "another"));
            jar.add(Cookie::new("third", "another"));
            let cookies = get_cookie_header(jar);
            for cookie in cookies {
                println!("{}", cookie);
                ctx.insert_http_header(http::header::SET_COOKIE, cookie);
            }
            let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api.projects")
                .fetch_one(pg_pool)
                .await?;
            Ok(count)
        }
    }
    
}
