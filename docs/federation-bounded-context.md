# Federation Bounded Context Documentation

## 1. Bounded Context Overview

**Bounded Context Name:** Federation

**Purpose:**  
The Federation Bounded Context is responsible for managing external identity provider integrations within the IAM system. It handles OAuth flows with third-party providers like Google, enabling users to authenticate using external accounts while maintaining system security and identity consistency.

**Primary Business Capability:**  
Exclusive ownership of external authentication provider integration and OAuth protocol handling. This includes OAuth flow orchestration, token exchange, external user profile mapping, and seamless integration with internal identity management.

**Out of Scope:**  
- Internal authentication and session management (handled by Authentication context)
- User identity creation and storage (handled by Identity context)
- Authorization and permissions (handled by separate contexts)
- Internal password-based authentication

## 2. Ubiquitous Language

| Term                  | Definition                                                                 | Notes |
|-----------------------|----------------------------------------------------------------------------|-------|
| Federation           | Integration with external identity providers for authentication           | Core business capability |
| OAuth                | Open standard for access delegation                                        | Protocol used for external auth |
| Provider             | External identity service (e.g., Google, Facebook)                        | Third-party authentication source |
| OAuth Flow           | Multi-step process: redirect → callback → token exchange → authentication | Standard OAuth 2.0 dance |
| Authorization Code   | Temporary code from OAuth provider for token exchange                     | Short-lived, single-use |
| CSRF State           | Security token to prevent cross-site request forgery                      | Random UUID for OAuth flow protection |
| External User        | User profile data from external provider                                  | Mapped to internal identity |
| Provider Mismatch    | Attempt to use different provider for existing account                    | Security violation |

## 3. Domain Model Documentation

### Value Objects

**GoogleUser**  
- **Description:** Represents a user profile retrieved from Google OAuth API.  
- **Invariant Rules:** Must contain valid email, subject ID, and verification status; email must be verified by Google.  
- **Business Examples:** {sub: "123456789", email: "user@gmail.com", email_verified: true, name: "John Doe"}

### Entities

**ExchangeTokens**  
- **Description:** Temporary storage for OAuth tokens during the exchange process.  
- **Invariant Rules:** Must contain valid access and refresh tokens, used for secure token handover.  
- **Business Examples:** {access_token: "ya29...", refresh_token: "1//..."} (stored temporarily in Redis).

## 4. Commands Documentation

(No explicit command objects - operations are handled through service methods)

## 5. Queries Documentation

(No explicit query objects - operations are handled through service methods)

## 6. Domain Events Documentation

(No domain events defined in current implementation - federation operations are synchronous)

## 7. Domain Services Documentation

**GoogleOAuthService**  
- **Business Capability:** Handles communication with Google OAuth API for token exchange and user profile retrieval.  
- **Inputs:** Authorization code from OAuth callback.  
- **Outputs:** GoogleUser profile data.  
- **Business Rules:** Must validate OAuth response, ensure email verification, handle API errors gracefully.

**TokenExchangeRepository**  
- **Business Capability:** Manages temporary storage of OAuth tokens during the federation flow.  
- **Inputs:** ExchangeTokens for storage, claim codes for retrieval.  
- **Outputs:** Stored tokens or confirmation of operations.  
- **Business Rules:** Must provide secure, time-limited storage with automatic cleanup.

## 8. Persistence & Repositories Documentation

**TokenExchange Aggregate**  
- **Persistence Responsibility:** Store OAuth tokens temporarily during federation flow.  
- **Consistency Rules:** Tokens must be retrievable by claim code, automatic expiration.  
- **Loading Strategy:** Load by claim code for token handover.  
- **Repository:** TokenExchangeRepository (Redis-based for temporary storage).

## 9. Application Layer Documentation

**GoogleFederationService**  
- **Responsibility:** Orchestrates the complete Google OAuth federation flow, from token exchange to identity creation/authentication.  
- **Flow Description:** Exchange code → Validate user → Check existing identity → Create/update identity → Generate auth tokens → Create session.  
- **Transactional Boundaries:** Each federation operation is atomic, identity creation and session setup are coordinated.  
- **Error Handling Strategy:** Domain errors for OAuth/protocol violations, infrastructure errors for technical failures.

## 10. Interfaces / API Documentation

**GET /api/v1/auth/google**  
- **Purpose:** Initiate Google OAuth flow by redirecting to Google.  
- **Input Contract:** (No body - initiated from frontend).  
- **Output Contract:** HTTP 302 redirect to Google OAuth URL with CSRF state.  
- **Error Scenarios:** 500 for configuration errors.

**GET /api/v1/auth/google/callback**  
- **Purpose:** Handle OAuth callback from Google with authorization code.  
- **Input Contract:** Query params: code (authorization code), state (CSRF token).  
- **Output Contract:** HTTP 302 redirect to frontend with claim code.  
- **Error Scenarios:** Primarily HTTP 302 redirects to frontend with `error` and `message` query params on failure. Returns 500 only when required local configuration is missing.

**POST /api/v1/auth/google/claim**  
- **Purpose:** Complete federation by exchanging claim code for authentication tokens.  
- **Input Contract:** {code: string} (claim code from callback redirect).  
- **Output Contract:** {token: string, refresh_token: string}.  
- **Error Scenarios:** 400 for invalid/expired code, 500 for server errors.

## 11. Anti-Corruption Layer (ACL) Documentation

### 11.1 Context Relationship
**Consumer Context:** Federation  
**Provider Context:** Identity (for user account management), Authentication (for session/token management)  
**Relationship Type:** Downstream dependencies (Federation uses both Identity and Authentication)

### 11.2 Translation Rules
- **External GoogleUser → Internal Identity:** Maps Google profile to internal identity format.  
- **Transformation Rules:** Google email/sub → Identity email/ID, generate secure placeholder password, set provider to Google.

### 11.3 Failure Handling
- **External Failure:** Google OAuth API unavailable or invalid response.  
- **Internal Reaction:** Return federation error, log security events.  
- **Fallback Strategy:** Graceful degradation with user-friendly error messages.

## 12. Context Boundaries & Integration Map

**Upstream Contexts:** Identity (provides identity creation/management), Authentication (provides token/session services).  
**Downstream Contexts:** None (entry point for external authentication).  
**Published Language:** None (federation results in authentication tokens).  
**ACL Boundaries:** Identity and Authentication contexts protected via service interfaces.
