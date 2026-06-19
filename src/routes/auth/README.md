# Authentication Router

This module configures the router and defines the routing table for all authentication-related endpoints. 

---

## Route Mappings

All routes defined in this router are mounted under the `/auth` path prefix in the main application router:

| Method | Path | Handler Function | Middleware Protection | Description |
| :--- | :--- | :--- | :--- | :--- |
| `POST` | `/auth/sign-up` | `sign_up_handler` | None | Register a new user account |
| `POST` | `/auth/verify-email` | `verify_email_handler` | None | Verify account via email OTP |
| `POST` | `/auth/login` | `login_handler` | None | Log in and receive HTTP-only cookies |
| `POST` | `/auth/refresh` | `refresh_handler` | None | Rotate expired Access Token |
| `POST` | `/auth/forgot-password` | `forgot_password_handler` | None | Request password reset code (OTP) |
| `POST` | `/auth/reset-password` | `reset_password_handler` | None | Complete password reset |
| `POST` | `/auth/logout` | `logout_handler` | None (reads refresh token) | Log out of the current session |
