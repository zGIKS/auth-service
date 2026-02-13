# IAM Auth Service

Open-source IAM service built with Rust + Axum, designed with **Domain-Driven Design (DDD)**.

## Project Summary

This project covers the full identity and access lifecycle:

- User registration with email confirmation.
- Sign in/sign out with JWT and refresh tokens.
- Password recovery and reset.
- Google OAuth federation.
- Security controls (rate limiting, lockout, session invalidation).

## What Problem It Solves

It centralizes common authentication and identity workflows in a single service, while keeping business rules, infrastructure, and APIs clearly separated by domain.

## Architecture (DDD)

The service is organized into bounded contexts and layers:

- **Identity**: registration, email confirmation, password reset.
- **Authentication**: sign in, refresh, logout, token verification.
- **Federation**: external authentication (Google OAuth).
- **Messaging**: email delivery through SMTP.
- **Shared**: cross-cutting concerns (AppState, middleware, lockout, rate limiting, circuit breaker).

Each context follows `domain`, `application`, `infrastructure`, and `interfaces` layers.

## C4 Diagrams

### 1. Context Diagram

Shows the IAM system and its relationship with external actors/systems (user, frontend, Google OAuth, SMTP).

![IAM Context Diagram](docs/assets/iam-context-dark.png)

### 2. Container Diagram

Shows the main containers: Axum API, PostgreSQL, and Redis, and how they interact.

![IAM Container Diagram](docs/assets/iam-containers-dark.png)

### 3. Component Diagram

Shows the internal API components (interfaces, application services, repositories, shared module, and messaging module).

![IAM Component Diagram](docs/assets/iam-components-dark.png)

## Module Documentation

- `docs/authentication-bounded-context.md`
- `docs/identity-bounded-context.md`
- `docs/federation-bounded-context.md`
- `docs/messaging-bounded-context.md`
- `docs/shared-module.md`
- `docs/c4-auth-service.dsl`

## Requirements

- Rust 1.70+
- PostgreSQL
- Redis
- SMTP server

## Minimal `.env` Configuration

```env
PORT=8081
DATABASE_URL=postgres://user:password@localhost/auth_service
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key
SESSION_DURATION_SECONDS=900
REFRESH_TOKEN_DURATION_SECONDS=604800
PENDING_REGISTRATION_TTL_SECONDS=900
PASSWORD_RESET_TTL_SECONDS=900
LOCKOUT_THRESHOLD=5
LOCKOUT_DURATION_SECONDS=900
FRONTEND_URL=http://localhost:3000
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GOOGLE_REDIRECT_URI=http://localhost:8081/api/v1/auth/google/callback
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
```

## Run

```bash
cargo run
```

Swagger UI: `http://localhost:8081/swagger-ui/`

If you run the service in Docker and PostgreSQL/Redis are on your host machine, use `host.docker.internal` in `.env` instead of `localhost`.

## Tests

```bash
cargo test
```
