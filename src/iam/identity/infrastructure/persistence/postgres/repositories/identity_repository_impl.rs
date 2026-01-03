use crate::iam::identity::domain::{
    model::{
        aggregates::identity::Identity as DomainIdentity,
        value_objects::{
            auth_provider::AuthProvider, email::Email, identity_id::IdentityId, password::Password,
        },
    },
    repositories::identity_repository::IdentityRepository,
};
use crate::iam::identity::infrastructure::persistence::postgres::model::{
    ActiveModel, Entity as IdentityEntity, Column,
};
use crate::shared::domain::model::entities::auditable_model::AuditableModel;
use sea_orm::*;
use std::error::Error;
use std::str::FromStr;
use bcrypt::{hash, DEFAULT_COST};

pub struct IdentityRepositoryImpl {
    db: DatabaseConnection,
}

impl IdentityRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl IdentityRepository for IdentityRepositoryImpl {
    async fn save(&self, identity: DomainIdentity) -> Result<DomainIdentity, Box<dyn Error + Send + Sync>> {
        let hashed_password = hash(identity.password().value(), DEFAULT_COST)?;

        let active_model = ActiveModel {
            id: Set(identity.id().0),
            email: Set(identity.email().value().to_string()),
            password_hash: Set(hashed_password),
            provider: Set(identity.provider().to_string()),
            is_verified: Set(identity.is_verified()),
            created_at: Set(identity.audit().created_at),
            updated_at: Set(identity.audit().updated_at),
        };

        IdentityEntity::insert(active_model)
             .exec(&self.db)
             .await?;
        
        Ok(identity)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<DomainIdentity>, Box<dyn Error + Send + Sync>> {
        let model = IdentityEntity::find()
            .filter(Column::Email.eq(email.value()))
            .one(&self.db)
            .await?;

        match model {
            Some(m) => {
                let email = Email::new(m.email).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                let provider = AuthProvider::from_str(&m.provider).map_err(|e| Box::<dyn Error + Send + Sync>::from(e))?;
                
                let audit = AuditableModel {
                    created_at: m.created_at,
                    updated_at: m.updated_at,
                };

                Ok(Some(DomainIdentity::new(
                    IdentityId::from_uuid(m.id),
                    email,
                    Password::new(m.password_hash),
                    provider,
                    m.is_verified,
                    audit
                )))
            }
            None => Ok(None)
        }
    }
}