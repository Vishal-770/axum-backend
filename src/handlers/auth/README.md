# Authentication Handlers

This directory contains the Axum HTTP request handlers that translate HTTP requests (JSON request payloads, path parameters, and cookies) into service calls, and map service results back to HTTP responses (JSON response bodies, cookie headers, and status codes).

---

## Handlers Reference

### 1. Sign Up (`sign_up.rs`)
* **Endpoint**: `POST /auth/signup`
* **Request Body**: `SignUpRequestDto` (JSON containing `email`, `username`, `password`).
* **Response**: `201 Created` with a confirmation message.

### 2. Verify Email (`verify_email.rs`)
* **Endpoint**: `POST /auth/verify`
* **Request Body**: `VerifyEmailRequestDto` (JSON containing `email`, `otp`).
* **Response**: `200 OK` with a confirmation message.

### 3. Login (`login.rs`)
* **Endpoint**: `POST /auth/login`
* **Request Body**: `LoginRequestDto` (JSON containing `email`, `password`).
* **Headers**: Reads `User-Agent` and parses IP address.
* **Cookies Written**:
  - `access_token`: HTTP-only, Secure, SameSite=Lax, Path=/, expires in 15 minutes.
  - `refresh_token`: HTTP-only, Secure, SameSite=Lax, Path=/, expires in 7 days.
* **Response**: `200 OK` with user details.

### 4. Refresh Token (`refresh.rs`)
* **Endpoint**: `POST /auth/refresh`
* **Request Cookies**: Reads the `refresh_token` cookie.
* **Headers**: Reads `User-Agent` and parses IP address.
* **Cookies Written**: Re-writes updated `access_token` and `refresh_token` cookies.
* **Response**: `200 OK` on success. If rotation fails (or token reuse is detected), returns `401 Unauthorized`.

### 5. Forgot Password (`forgot_password.rs`)
* **Endpoint**: `POST /auth/forgot-password`
* **Request Body**: `ForgotPasswordRequestDto` (JSON containing `email`).
* **Response**: `200 OK` with `reset_token` (UUID/random string) inside a JSON payload:
  ```json
  {
    "reset_token": "xxxx-xxxx-xxxx-xxxx"
  }
  ```
  *(Note: The actual OTP is sent to the user's email).*

### 6. Reset Password (`reset_password.rs`)
* **Endpoint**: `POST /auth/reset-password`
* **Request Body**: `ResetPasswordRequestDto` (JSON containing `reset_token`, `otp`, `new_password`).
* **Response**: `200 OK` with a success message.

### 7. Logout (`logout.rs`)
* **Endpoint**: `POST /auth/logout`
* **Request Cookies**: Reads `refresh_token` to revoke it in the database.
* **Cookies Written**: Clears `access_token` and `refresh_token` by setting their max-age to 0.
* **Response**: `200 OK` (Always succeeds/is idempotent).
