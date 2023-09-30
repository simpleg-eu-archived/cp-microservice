pub trait Token {
    fn can_execute(&self, action: &str) -> bool;
    fn user_id(&self) -> &str;
    fn org_id(&self) -> Option<&str>;
}
