use rust_backend::app::app_router;

#[tokio::main]
 async  fn main() {

    let listener=tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on port 3000");
    axum::serve(listener,app_router()).await.unwrap();

}




