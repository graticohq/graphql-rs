
// use async_graphql::{Context, Result};
// use sqlx::PgPool;


// pub async fn get_all<'a>(ctx: &'a Context<'_>) -> Result<i64>  {
//     let pool = ctx.data::<PgPool>()?;
//     let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api.projects")
//         .fetch_one(pool)
//         .await?;
//         let b = count;
//     Ok(b)
// }
