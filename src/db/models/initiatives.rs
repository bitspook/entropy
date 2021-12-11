use diesel::{data_types::PgInterval, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use serde::Serialize;

use crate::db::schema::*;

#[derive(Insertable, Queryable, Debug, Serialize)]
#[table_name = "initiatives"]
pub struct Initiative {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub source: Option<String>,
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
    pub iteration_interval: Option<PgInterval>,
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
    pub iteration_interval: Option<PgInterval>,
}

impl Initiative {
    pub fn count_initiatives(conn: &PgConnection) -> anyhow::Result<i64> {
        use crate::db::schema::initiatives::dsl::*;

        initiatives
            .count()
            .inner_join(goals::table)
            .get_result(conn)
            .map_err(anyhow::Error::from)
    }

    pub fn count_rfcs(conn: &PgConnection) -> anyhow::Result<i64> {
        use crate::db::schema::goals::dsl::*;

        initiatives::table
            .count()
            .left_join(goals)
            .filter(slug.is_null())
            .get_result(conn)
            .map_err(anyhow::Error::from)
    }
}
