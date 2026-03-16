use bevy::prelude::*;

#[derive(Resource, Default, Clone)]
pub struct GameName(pub String);

impl GameName {
    pub fn display(&self) -> &str {
        if self.0.is_empty() { "My City" } else { &self.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_returns_my_city_when_empty() {
        let name = GameName(String::new());
        assert_eq!(name.display(), "My City");
    }

    #[test]
    fn display_returns_custom_name() {
        let name = GameName("Springfield".to_string());
        assert_eq!(name.display(), "Springfield");
    }
}
