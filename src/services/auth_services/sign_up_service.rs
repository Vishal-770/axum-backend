pub async fn sign_up(email: String,user_name:String ,password: String)->Result<(),()>{
    println!("Sign up service called with email: {}, username: {} and password: {}", email,user_name,password);
   Ok(())
}
