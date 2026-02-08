# Authentication Bounded Context Documentation

## 1. Bounded Context Overview

**Bounded Context Name:** Authentication

**Purpose:**  
The Authentication Bounded Context is responsible for managing user authentication sessions and token-based access control within the IAM system. It handles the complete authentication lifecycle including sign-in, token refresh, logout, and session management, ensuring secure and controlled access to system resources.

**Primary Business Capability:**  
Exclusive ownership of authentication state management, session control, and token validation. This includes JWT token generation/validation, refresh token handling, session tracking, and secure logout with session invalidation.

**Out of Scope:**  
- User identity creation and management (handled by Identity context)
- Authorization and permissions (handled by separate contexts)
- Social media federation and OAuth (handled by Federation context)
- Password storage and validation (handled by Identity context)

## 2. Ubiquitous Language

| Term                  | Definition                                                                 | Notes |
|-----------------------|----------------------------------------------------------------------------|-------|
| Authentication       | The process of verifying user identity and establishing a session         | Core business capability |
| Session              | An active user login state with associated tokens and metadata           | Managed per user |
| Token                | A JWT access token for API authorization                                  | Short-lived, stateless |
| RefreshToken         | A long-lived token for obtaining new access tokens                        | Stored securely |
| Claims               | User identity information encoded in JWT tokens                           | Includes user ID, expiration |
| JTI                  | JWT ID - unique identifier for token revocation                           | Used for session tracking |
| Signin               | User authentication attempt with credentials                              | Initiates session |
| Logout               | User-initiated session termination                                        | Invalidates all tokens |
| AccountLockout       | Temporary access restriction after failed authentication attempts         | Security measure |

## 3. Domain Model Documentation

### Value Objects

**Token**  
- **Description:** Represents a JWT access token used for API authorization.  
- **Invariant Rules:** Must be valid JWT format, cryptographically signed, contains user claims.  
- **Business Examples:** eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

**RefreshToken**  
- **Description:** Represents a long-lived token for obtaining new access tokens without re-authentication.  
- **Invariant Rules:** Must be unique, securely stored, time-limited with TTL.  
- **Business Examples:** Randomly generated secure strings with expiration.

**Claims**  
- **Description:** User identity and authorization information encoded in JWT tokens.  
- **Invariant Rules:** Must include subject (user ID), expiration time, issued at time, and unique JTI.  
- **Business Examples:** {sub: "uuid", exp: 1640995200, jti: "unique-id"}

### Entities

**Session**  
- **Description:** Represents an active authentication session for a user.  
- **Invariant Rules:** Each user can have only one active session, must be tracked for revocation.  
- **Business Examples:** User login state with associated tokens and metadata.

## 4. Commands Documentation

**SigninCommand**  
- **Intent:** Authenticate user credentials and establish a new session.  
- **Required Data:** Email, Password, IP Address (optional).  
- **Business Rules:** Credentials must be valid, account must not be locked, user must exist.  
- **Possible Rejections:** InvalidCredentials, AccountLocked, UserNotFound.

**RefreshTokenCommand**  
- **Intent:** Obtain new access token using valid refresh token.  
- **Required Data:** RefreshToken.  
- **Business Rules:** Refresh token must be valid and not expired, associated user must still exist.  
- **Possible Rejections:** InvalidToken, TokenExpired, UserNotFound.

**LogoutCommand**  
- **Intent:** Terminate user session and invalidate all associated tokens.  
- **Required Data:** RefreshToken.  
- **Business Rules:** Refresh token must be present and the associated session must be invalidated.  
- **Possible Rejections:** InvalidToken.

## 5. Queries Documentation

**VerifyTokenQuery**  
- **Information Requested:** Token validity and associated user claims.  
- **Filters:** JWT token string.  
- **Constraints:** Token must be properly formatted and signed.  
- **Returned Data:** Claims object with user information.

## 6. Domain Events Documentation

(No domain events defined in current implementation - authentication state changes are handled internally)

## 7. Domain Services Documentation

**TokenService**  
- **Business Capability:** Handles JWT token generation, validation, and refresh token creation.  
- **Inputs:** User ID for token generation, token strings for validation.  
- **Outputs:** Signed tokens or validated claims.  
- **Business Rules:** Tokens must be cryptographically secure, properly signed, and contain required claims.

**SessionRepository**  
- **Business Capability:** Manages authentication session state and token storage.  
- **Inputs:** User ID, tokens, TTL values.  
- **Outputs:** Session data or confirmation of operations.  
- **Business Rules:** Must support atomic operations, proper TTL handling, and secure token storage.

**AuthenticationCommandService**  
- **Business Capability:** Orchestrates authentication operations including signin, refresh, and logout.  
- **Inputs:** Authentication commands.  
- **Outputs:** Token pairs or operation confirmations.  
- **Business Rules:** Must enforce security policies, handle account lockout, and coordinate with identity verification.

**AuthenticationQueryService**  
- **Business Capability:** Provides token verification and claims extraction.  
- **Inputs:** Token strings.  
- **Outputs:** Validated claims.  
- **Business Rules:** Must validate token integrity and expiration.

## 8. Persistence & Repositories Documentation

**Session Aggregate**  
- **Persistence Responsibility:** Store active session state, refresh tokens, and revocation data.  
- **Consistency Rules:** User-session uniqueness, atomic token operations.  
- **Loading Strategy:** Load by user ID or refresh token.  
- **Repository:** SessionRepository (Redis-based for performance).

## 9. Application Layer Documentation

**AuthenticationCommandServiceImpl**  
- **Responsibility:** Implements authentication business logic, coordinates with identity verification, and manages session state.  
- **Flow Description:** Validate credentials → Check lockout → Generate tokens → Create session → Return token pair.  
- **Transactional Boundaries:** Each authentication operation is atomic, session operations use Redis transactions.  
- **Error Handling Strategy:** Domain errors for business violations, infrastructure errors for technical failures.

## 10. Interfaces / API Documentation

**Note:** Google OAuth endpoints (/api/v1/auth/google, /api/v1/auth/google/callback, /api/v1/auth/google/claim) are handled by the Federation Bounded Context, not Authentication.

**POST /api/v1/auth/sign-in**  
- **Purpose:** Authenticate user and establish session.  
- **Input Contract:** {email: string, password: string}.  
- **Output Contract:** {token: string, refresh_token: string}.  
- **Error Scenarios:** 400 for validation errors, 401 for invalid credentials.

**POST /api/v1/auth/refresh-token**  
- **Purpose:** Obtain new access token using refresh token.  
- **Input Contract:** {refresh_token: string}.  
- **Output Contract:** {token: string, refresh_token: string}.  
- **Error Scenarios:** 400 for validation errors, 401 for invalid/expired refresh token.

**POST /api/v1/auth/logout**  
- **Purpose:** Terminate user session and invalidate tokens.  
- **Input Contract:** {refresh_token: string}.  
- **Output Contract:** HTTP 200 with empty body on success.  
- **Error Scenarios:** 400 for validation errors, 500 for infrastructure/internal errors.

**GET /api/v1/auth/verify**  
- **Purpose:** Verify token validity and get claims.  
- **Input Contract:** Query param `token` (non-empty string).  
- **Output Contract:** {is_valid: boolean, sub: uuid, error?: string}.  
- **Error Scenarios:** 400 for invalid input format. Business validation failures return HTTP 200 with `is_valid: false`.

## 11. Anti-Corruption Layer (ACL) Documentation

### 11.1 Context Relationship
**Consumer Context:** Authentication  
**Provider Context:** Identity  
**Relationship Type:** Downstream dependency (Authentication uses Identity for credential verification)

### 11.2 Translation Rules
- **Internal SigninCommand → External IdentityFacade:** Translates authentication requests to identity verification calls.  
- **Transformation Rules:** Authentication email/password → Identity verification request, error code translation.

### 11.3 Failure Handling
- **External Failure:** Identity service unavailable or credential verification failure.  
- **Internal Reaction:** Return authentication failure, log security events.  
- **Fallback Strategy:** Graceful degradation with appropriate error messages.

## 12. Context Boundaries & Integration Map

**Upstream Contexts:** Identity (provides verified user identities), Federation (may provide alternative authentication methods).  
**Downstream Contexts:** Any context requiring authenticated access.  
**Published Language:** None (internal authentication state).  
**ACL Boundaries:** Identity context protected via IdentityFacade.
