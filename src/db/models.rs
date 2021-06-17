use super::schema::meetup_groups;

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
    pub member_count: i32,
    pub photo: String
}

#[derive(Debug, Insertable)]
#[table_name="meetup_groups"]
pub struct NewMeetupGroup {
    pub id: String,
    pub name: String,
    pub link: String,
    pub description: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub is_private: bool,
    pub member_count: i32,
    pub photo: String
}
