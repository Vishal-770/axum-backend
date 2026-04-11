use std::env;
use rust_backend::app::app_router;
use rust_backend::database::db_pool::connect_db;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool=connect_db(&db_url).await;
    let app=app_router(pool);
    let listener=tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on port 3000");
    axum::serve(listener,app).await.unwrap();

}




