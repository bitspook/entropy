use crate::db::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "groups"]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub desc_format: String,
    pub photos: Vec<String>,
    pub source: Option<String>,
    pub source_link: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Group {
    pub fn with_slug(
        slug_to_find: &str,
        conn: &PgConnection,
    ) -> Result<Self, diesel::result::Error> {
        use crate::db::schema::groups::dsl::*;

        groups.filter(slug.eq(slug_to_find)).first(conn)
    }
}

#[derive(Insertable, Debug)]
#[table_name = "groups"]
pub struct NewGroup {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    // TODO Change desc_format to an Enum
    pub desc_format: String,
    pub source: Option<String>,
    pub source_link: Option<String>,
}
