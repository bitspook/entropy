#[derive(Queryable)]
pub struct MeetupGroup {
    pub id: String,
    pub name: String,
    pub link: String,
    pub description: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub is_private: bool,
    pub member_count: u32,
    pub photo: String
}
