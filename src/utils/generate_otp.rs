use rand::Rng;

pub async fn generate_otp() -> String {
    let mut rng = rand::rng();
    let otp: u32 = rng.random_range(100000..=999999);
    otp.to_string()
}

