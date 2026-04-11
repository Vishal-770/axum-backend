use crate::database::db_state::AppState;

pub async fn sign_up(
    email: String,
    user_name: String,
    password: String,
    state: AppState,
) -> Result<(), ()> {
    // let user=sqlx::query_as!(
    //     User,
    // )
    println!(
        "Sign up service called with email: {}, username: {} and password: {}, state{:?}",
        email, user_name, password, state
    );
    Ok(())
}
