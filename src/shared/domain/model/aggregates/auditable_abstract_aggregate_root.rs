use crate::shared::domain::model::entities::auditable_model::AuditableModel;

#[derive(Debug, Clone)]
pub struct AuditableAbstractAggregateRoot {
    pub audit: AuditableModel,
    // Aquí irían los eventos de dominio si estuviéramos implementando un mecanismo genérico
    // pub domain_events: Vec<DomainEvent>,
}

impl AuditableAbstractAggregateRoot {
    pub fn new() -> Self {
        Self {
            audit: AuditableModel::new(),
        }
    }

    pub fn update_audit(&mut self) {
        self.audit.update();
    }
}

impl Default for AuditableAbstractAggregateRoot {
    fn default() -> Self {
        Self::new()
    }
}
