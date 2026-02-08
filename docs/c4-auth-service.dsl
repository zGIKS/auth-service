workspace "IAM Auth Service" "C4 model for the open-source IAM auth-service project" {

    model {
        endUser = person "User" "User who signs up, signs in, and manages credentials."

        frontend = softwareSystem "Frontend Application" "Web or mobile client that consumes IAM APIs." {
            tags "External"
        }

        googleOAuth = softwareSystem "Google OAuth" "External identity provider for federation login." {
            tags "External"
        }

        smtpServer = softwareSystem "SMTP Server" "External email delivery provider." {
            tags "External"
        }

        iam = softwareSystem "IAM Auth Service" "Rust + Axum service implementing DDD bounded contexts: Identity, Authentication, Federation, Messaging, and Shared module." {
            api = container "IAM API" "Single deployable Axum application exposing REST endpoints and OpenAPI." "Rust, Axum" {
                identityInterfaces = component "Identity Interfaces" "REST controllers/resources for sign-up, confirm registration, forgot/reset password." "Rust module: iam::identity::interfaces::rest"
                authenticationInterfaces = component "Authentication Interfaces" "REST controllers/resources for sign-in, refresh, logout, verify token." "Rust module: iam::authentication::interfaces::rest"
                federationInterfaces = component "Federation Interfaces" "REST controllers/resources for Google OAuth redirect/callback/claim flow." "Rust module: iam::federation::interfaces::rest"

                identityApplication = component "Identity Application Services" "Implements identity use cases, orchestrates repositories and notifications." "IdentityCommandServiceImpl"
                authenticationApplication = component "Authentication Application Services" "Implements signin/logout/refresh and token verification use cases." "AuthenticationCommandServiceImpl, AuthenticationQueryServiceImpl"
                federationApplication = component "Federation Application Services" "Implements Google federation flow and account linking/creation." "GoogleFederationService"

                messagingModule = component "Messaging Module" "Email facade + command service + SMTP sender implementation." "messaging::*"
                sharedModule = component "Shared Module" "Cross-cutting concerns: AppState, middleware, lockout, rate limiting, circuit breaker, error response." "shared::*"

                jwtTokenService = component "JWT Token Service" "Generates/verifies access and refresh tokens." "JwtTokenService"
                identityRepository = component "Identity Repository" "PostgreSQL repository for Identity aggregate." "IdentityRepositoryImpl"
                redisRepositories = component "Redis Repositories" "Session, pending identity, password reset token, token exchange repositories." "RedisSessionRepository, PendingIdentityRepositoryImpl, PasswordResetTokenRepositoryImpl, TokenExchangeRepositoryImpl"
            }

            postgres = container "PostgreSQL" "Primary relational storage for verified identities." "PostgreSQL" {
                tags "Database"
            }
            redis = container "Redis" "Storage for sessions, TTL tokens, lockout/rate-limit state, and exchange codes." "Redis" {
                tags "Database"
            }
        }

        endUser -> frontend "Uses"
        frontend -> api "Calls REST API (JSON/HTTPS)"

        api -> postgres "Reads/writes identity data"
        api -> redis "Reads/writes session and temporary security state"
        api -> googleOAuth "OAuth 2.0 code exchange and user info"
        api -> smtpServer "Sends verification and password reset emails"

        identityInterfaces -> identityApplication "Invokes use cases"
        authenticationInterfaces -> authenticationApplication "Invokes use cases"
        federationInterfaces -> federationApplication "Invokes use cases"

        identityApplication -> identityRepository "Persists identities"
        identityApplication -> redisRepositories "Stores pending/reset artifacts"
        identityApplication -> messagingModule "Sends domain notifications"

        authenticationApplication -> identityRepository "Verifies credentials via identity access"
        authenticationApplication -> jwtTokenService "Generates/verifies tokens"
        authenticationApplication -> redisRepositories "Stores sessions and revocation state"
        authenticationApplication -> sharedModule "Uses lockout and security policies"

        federationApplication -> googleOAuth "Exchanges authorization code"
        federationApplication -> identityRepository "Loads/creates identity (Google provider)"
        federationApplication -> jwtTokenService "Generates service tokens"
        federationApplication -> redisRepositories "Stores one-time token exchange codes"

        messagingModule -> smtpServer "SMTP delivery"

        identityRepository -> postgres "CRUD operations"
        redisRepositories -> redis "CRUD + TTL operations"

        identityInterfaces -> sharedModule "Uses AppState, middleware, errors"
        authenticationInterfaces -> sharedModule "Uses AppState, middleware, errors"
        federationInterfaces -> sharedModule "Uses AppState, middleware, errors"
    }

    views {
        systemContext iam "iam-context" "System Context - IAM Auth Service" {
            include endUser
            include frontend
            include googleOAuth
            include smtpServer
            include iam
            autolayout lr
        }

        container iam "iam-containers" "Container Diagram - IAM Auth Service" {
            include endUser
            include frontend
            include googleOAuth
            include smtpServer
            include api
            include postgres
            include redis
            autolayout lr
        }

        component api "iam-components" "Component Diagram - IAM API" {
            include *
            exclude endUser
            exclude frontend
            autolayout lr
        }

        styles {
            element "Person" {
                background "#08427b"
                color "#ffffff"
                shape person
            }
            element "Software System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Component" {
                background "#85bbf0"
                color "#000000"
            }
            element "Database" {
                shape cylinder
                background "#2e7d32"
                color "#ffffff"
            }
            element "External" {
                background "#999999"
                color "#ffffff"
                border "dashed"
            }
        }
    }
}
