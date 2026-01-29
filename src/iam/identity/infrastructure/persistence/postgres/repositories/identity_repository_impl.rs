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
    ActiveModel, Column, Entity as IdentityEntity,
};
use crate::shared::domain::model::entities::auditable_model::AuditableModel;
use sea_orm::*;
use std::error::Error;
use std::str::FromStr;

pub struct IdentityRepositoryImpl {
    db: DatabaseConnection,
}

impl IdentityRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl IdentityRepository for IdentityRepositoryImpl {
    async fn save(
        &self,
        identity: DomainIdentity,
    ) -> Result<DomainIdentity, Box<dyn Error + Send + Sync>> {
        // Password is already hashed by the service layer
        let password_hash_value = identity.password().value().to_string();

        let active_model = ActiveModel {
            id: Set(identity.id().0),
            email: Set(identity.email().value().to_string()),
            password_hash: Set(password_hash_value),
            auth_provider: Set(identity.provider().to_string()),
            created_at: Set(identity.audit().created_at.into()),
            updated_at: Set(identity.audit().updated_at.into()),
        };

        IdentityEntity::insert(active_model).exec(&self.db).await?;

        Ok(identity)
    }

    async fn find_by_email(
        &self,
        email: &Email,
    ) -> Result<Option<DomainIdentity>, Box<dyn Error + Send + Sync>> {
        let model = IdentityEntity::find()
            .filter(Column::Email.eq(email.value()))
            .one(&self.db)
            .await?;

        match model {
            Some(m) => {
                let email =
                    Email::new(m.email).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                let provider = AuthProvider::from_str(&m.auth_provider)
                    .map_err(Box::<dyn Error + Send + Sync>::from)?;

                let audit = AuditableModel {
                    created_at: m.created_at.into(),
                    updated_at: m.updated_at.into(),
                };

                Ok(Some(DomainIdentity::new(
                    IdentityId::from_uuid(m.id),
                    email,
                    Password::new(m.password_hash).map_err(Box::<dyn Error + Send + Sync>::from)?,
                    provider,
                    audit,
                )))
            }
            None => Ok(None),
        }
    }
}
