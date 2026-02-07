# IAM Bounded Context Documentation

## 1. Bounded Context Overview

**Bounded Context Name:** IAM (Identity and Access Management)

**Purpose:**  
The IAM Bounded Context is responsible for managing user identities, authentication, and federation within the system. It provides secure mechanisms for user registration, login, password management, and integration with external identity providers.

**Primary Business Capability:**  
Exclusive ownership of user identity lifecycle, authentication processes, and secure access control.

**Out of Scope:**  
- Business logic specific to other domains (e.g., order processing, inventory management)
- Authorization policies beyond basic authentication
- User profile data management (limited to identity basics)
- Payment processing or financial transactions

## 2. Ubiquitous Language

| Term | Definition | Notes |
|------|------------|-------|
| Identity | A unique user entity with email, password, and provider information | Core aggregate representing a user |
| Authentication | The process of verifying user identity and granting access | Includes signin, logout, token management |
| Federation | Integration with external identity providers (e.g., Google) | Supports OAuth-based login |
| Verification Token | A secure token for email confirmation or password reset | Time-limited and single-use |
| Pending Identity | A temporary state for unconfirmed user registrations | Expires after TTL |
| Session | An authenticated user session with access tokens | Managed via Redis |
| Password Reset Token | A token for secure password change process | Separate from verification tokens |

## 3. Domain Model Documentation

### Value Objects
- **Email**: Represents a validated email address with MX record checking
- **Password**: Represents a password with length validation (6-72 characters)
- **IdentityId**: A unique UUID identifier for identities
- **AuthProvider**: Enum for authentication providers (Local, Google)
- **VerificationToken**: Secure token for email verification
- **PendingIdentity**: Temporary identity state during registration

### Entities
- **Identity**: The main entity representing a user with email, password, provider, and audit information

### Aggregates
- **Identity Aggregate**: Root aggregate containing Identity entity and related value objects

### Domain Invariants
- Email must be unique across all identities
- Password must meet security requirements
- Verification tokens expire after configured TTL
- Pending identities are automatically cleaned up

## 4. Commands Documentation

### RegisterIdentityCommand
**Intent:** Register a new user identity in the system  
**Required Data:** email, password, auth_provider  
**Business Rules:** Email must be valid and unique, password meets requirements  
**Possible Rejections:** Invalid email format, duplicate email, weak password

### ConfirmRegistrationCommand
**Intent:** Confirm email address using verification token  
**Required Data:** verification_token  
**Business Rules:** Token must be valid and not expired  
**Possible Rejections:** Invalid token, expired token

### RequestPasswordResetCommand
**Intent:** Initiate password reset process for an identity  
**Required Data:** email  
**Business Rules:** Email must exist in system  
**Possible Rejections:** Email not found

### ResetPasswordCommand
**Intent:** Change password using reset token  
**Required Data:** reset_token, new_password  
**Business Rules:** Token valid, new password meets requirements  
**Possible Rejections:** Invalid token, weak password

### SigninCommand
**Intent:** Authenticate user and create session  
**Required Data:** email, password, ip_address (optional)  
**Business Rules:** Credentials must be valid  
**Possible Rejections:** Invalid credentials, account locked

### LogoutCommand
**Intent:** End user session  
**Required Data:** session_id  
**Business Rules:** Session must exist  
**Possible Rejections:** Invalid session

### RefreshTokenCommand
**Intent:** Generate new access token using refresh token  
**Required Data:** refresh_token  
**Business Rules:** Refresh token must be valid  
**Possible Rejections:** Invalid refresh token

## 5. Queries Documentation

### ConfirmEmailQuery
**Information Requested:** Email confirmation status  
**Filters:** verification_token  
**Constraints:** Token must be valid format (min 32 chars)  
**Returned Data:** Confirmation result

## 6. Domain Events Documentation

### IdentityRegisteredEvent
**Business Meaning:** A new identity has been successfully registered  
**Triggered When:** After successful identity registration  
**Consumers:** Notification service, audit logging  
**Data Carried:** identity_id, occurred_on

## 7. Domain Services Documentation

### IdentityCommandService
**Business Capability:** Orchestrates identity-related business operations  
**Inputs:** Various commands (register, confirm, reset password)  
**Outputs:** Success/failure results  
**Business Rules:** Enforces domain invariants, coordinates with repositories

### NotificationService
**Business Capability:** Sends email notifications for identity events  
**Inputs:** Email content, recipient  
**Outputs:** Send result  
**Business Rules:** Ensures secure email delivery

### SessionInvalidationService
**Business Capability:** Manages session cleanup on password changes  
**Inputs:** Identity ID  
**Outputs:** Invalidation result  
**Business Rules:** Invalidates all active sessions for security

## 8. Persistence & Repositories Documentation

### Identity Aggregate
**Persistence Responsibility:** Stores complete identity information  
**Consistency Rules:** Email uniqueness enforced at database level  
**Loading Strategy:** Load by ID or email  
**Repository:** IdentityRepository

### PendingIdentity
**Persistence Responsibility:** Temporary storage for unconfirmed registrations  
**Consistency Rules:** TTL-based expiration  
**Loading Strategy:** Load by token  
**Repository:** PendingIdentityRepository

### PasswordResetToken
**Persistence Responsibility:** Secure storage of reset tokens  
**Consistency Rules:** TTL-based expiration  
**Loading Strategy:** Load by token  
**Repository:** PasswordResetTokenRepository

## 9. Application Layer Documentation

### IdentityCommandServiceImpl
**Responsibility:** Implements identity command handling with dependency injection  
**Flow Description:** Validates commands, orchestrates domain services, persists changes  
**Transactional Boundaries:** Each command is atomic  
**Error Handling Strategy:** Domain errors converted to application responses

### MessagingFacadeImpl
**Responsibility:** Anti-corruption layer for messaging integration  
**Flow Description:** Translates domain events to messaging commands  
**Transactional Boundaries:** Event-driven, no direct transactions  
**Error Handling Strategy:** Circuit breaker pattern for resilience

## 10. Interfaces / API Documentation

### Identity Management

#### POST /api/v1/identity/sign-up
**Purpose:** Register new identity  
**Input Contract:** RegisterIdentityRequest (email, password)  
**Output Contract:** RegisterIdentityResponse or ErrorResponse  
**Error Scenarios:** Validation errors, duplicate email

#### GET /api/v1/identity/confirm-registration
**Purpose:** Confirm email registration  
**Input Contract:** Query params (token)  
**Output Contract:** Redirect or error page  
**Error Scenarios:** Invalid token, expired token

#### POST /api/v1/identity/forgot-password
**Purpose:** Request password reset  
**Input Contract:** RequestPasswordResetRequest (email)  
**Output Contract:** RequestPasswordResetResponse  
**Error Scenarios:** Email not found

#### POST /api/v1/identity/reset-password
**Purpose:** Reset password  
**Input Contract:** ResetPasswordRequest (token, new_password)  
**Output Contract:** ResetPasswordResponse  
**Error Scenarios:** Invalid token, weak password

### Authentication

#### POST /api/v1/auth/sign-in
**Purpose:** Authenticate user  
**Input Contract:** SigninResource (email, password)  
**Output Contract:** TokenResponse (access_token, refresh_token)  
**Error Scenarios:** Invalid credentials

#### POST /api/v1/auth/logout
**Purpose:** End session  
**Input Contract:** LogoutResource (session_id)  
**Output Contract:** Success response  
**Error Scenarios:** Invalid session

#### POST /api/v1/auth/refresh-token
**Purpose:** Refresh access token  
**Input Contract:** RefreshTokenResource (refresh_token)  
**Output Contract:** TokenResponse  
**Error Scenarios:** Invalid refresh token

#### GET /api/v1/auth/verify
**Purpose:** Verify token validity  
**Input Contract:** VerifyTokenResource (token)  
**Output Contract:** VerifyTokenResponse (valid, claims)  
**Error Scenarios:** Invalid token

#### GET /api/v1/auth/google
**Purpose:** Initiate Google OAuth login  
**Input Contract:** None  
**Output Contract:** Redirect to Google  
**Error Scenarios:** OAuth configuration errors

#### GET /api/v1/auth/google/callback
**Purpose:** Handle Google OAuth callback  
**Input Contract:** Query params (code, state)  
**Output Contract:** TokenResponse or redirect  
**Error Scenarios:** OAuth errors, invalid state

## 11. Anti-Corruption Layer (ACL) Documentation

### 11.1 Context Relationship
**Consumer Context:** IAM  
**Provider Context:** Messaging  
**Relationship Type:** Downstream (IAM publishes events to Messaging)

### 11.2 Translation Rules
**Domain Event → Messaging Command**  
IdentityRegisteredEvent → SendWelcomeEmailCommand  
Translation Rules: Extract email from identity, format welcome message

### 11.3 Failure Handling
**External Failure:** Email service unavailable  
**Internal Reaction:** Log error, do not fail registration  
**Fallback Strategy:** Retry with circuit breaker

## 12. Context Boundaries & Integration Map

### Upstream Contexts
- **User Interface**: Consumes authentication APIs
- **API Gateway**: Routes requests to IAM endpoints

### Downstream Contexts
- **Messaging**: Receives identity events for notifications
- **Audit**: Receives events for compliance logging

### Integration Points
- REST APIs for external consumers
- Domain events for internal integration
- ACL protects from external model changes

## 13. Common Pitfalls

- ❌ Storing sensitive data in domain objects
- ❌ Mixing authentication logic with business domains
- ❌ Exposing internal IDs in APIs
- ❌ Hardcoding email templates in domain
- ❌ Sharing identity models across contexts

## 14. Documentation Maintenance Rules

- Update documentation with every domain change
- Code reviews must verify documentation accuracy
- Domain experts validate ubiquitous language usage
- Automated tests ensure API contracts remain stable</content>
<parameter name="filePath">/home/giks/Documents/IAM-service/auth-service-main/docs/IAM_Bounded_Context_Documentation.md
