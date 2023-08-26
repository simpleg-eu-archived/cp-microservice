pub trait Token {
    fn can_execute(&self, action: &str) -> bool;
}
