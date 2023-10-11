pub trait SecretsManager {
    fn get(&self, id: &str) -> Option<String>;
}
