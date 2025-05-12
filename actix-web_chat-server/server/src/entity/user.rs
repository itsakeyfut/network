#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub current_room: Option<String>,
}
