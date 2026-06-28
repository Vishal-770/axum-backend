use std::env;
use rust_backend::app::app_router;
use rust_backend::database::db_pool::connect_db;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool=connect_db(&db_url).await;

    // Run database migrations on startup
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    let app=app_router(pool, redis_conn);
    let listener=tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on port 3000");
    axum::serve(listener,app).await.unwrap();
}




