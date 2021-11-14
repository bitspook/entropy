use diesel::data_types::PgInterval;

use crate::db::schema::*;

#[derive(Insertable, Queryable, Debug)]
#[table_name = "initiatives"]
pub struct Initiative {
    pub slug: String,
    pub title: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub desc_format: String,
}

#[derive(Insertable, Queryable, Debug)]
#[table_name = "goals"]
pub struct Goal {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub desc_format: String,
    pub initiative_slug: String,
    pub target: i32,
    pub iteration_interval: Option<PgInterval>
}

#[derive(Insertable, Queryable, Debug)]
#[table_name = "goals"]
pub struct NewGoal {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub desc_format: String,
    pub initiative_slug: Option<String>,
    pub target: i32,
    pub iteration_interval: Option<PgInterval>
}
