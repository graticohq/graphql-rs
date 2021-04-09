
pub mod graphql {
    
    use async_graphql::{Context, FieldResult, Object};
    use cookie::{Cookie, CookieJar, Key, ParseError};

    
    use sqlx::postgres::{PgPool};

    #[derive(Debug, Clone)]
    pub struct GraphQLContext<'b> {
        pub db_pool: &'b PgPool,
         pub cookie_jar:  &'b CookieJar
    }


    pub struct QueryRoot;
    #[Object]
    impl QueryRoot {
        async fn posts<'a>(&self, ctx: &Context<'_>) -> FieldResult<i64> {
            let c = ctx.data_unchecked::<GraphQLContext>();
            c.cookie_jar.to_owned().add(Cookie::new("second", "another"));
            let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api.projects")
                .fetch_one(&*c.db_pool)
                .await?;
            Ok(count)
        }
    }

}
