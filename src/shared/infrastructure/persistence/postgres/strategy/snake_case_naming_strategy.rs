/// Estrategia de nombrado para emular SnakeCaseWithPluralizedTablePhysicalNamingStrategy de JPA.
/// En Rust/SeaORM, los nombres de tablas se definen usualmente con atributos `#[sea_orm(table_name = "...")]`.
/// Esta estructura sirve como utilidad o referencia a la convención.
pub struct SnakeCaseWithPluralizedTablePhysicalNamingStrategy;

impl SnakeCaseWithPluralizedTablePhysicalNamingStrategy {
    /// Convierte un nombre (e.g. "UserProfile") a snake_case pluralizado (e.g. "user_profiles").
    /// Nota: Esta es una implementación simplificada.
    pub fn to_physical_table_name(entity_name: &str) -> String {
        let snake = Self::to_snake_case(entity_name);
        Self::pluralize(&snake)
    }

    fn to_snake_case(s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }

    fn pluralize(s: &str) -> String {
        // Regla muy básica: agregar 's'.
        // En producción se usaría una crate como 'heck' o 'inflector'.
        format!("{}s", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naming() {
        assert_eq!(
            SnakeCaseWithPluralizedTablePhysicalNamingStrategy::to_physical_table_name("User"),
            "users"
        );
        assert_eq!(
            SnakeCaseWithPluralizedTablePhysicalNamingStrategy::to_physical_table_name(
                "UserProfile"
            ),
            "user_profiles"
        );
    }
}
