table! {
    meetup_events (id) {
        id -> Int4,
        slug -> Varchar,
        group_slug -> Varchar,
        title -> Varchar,
        description -> Nullable<Varchar>,
        start_time -> Timestamp,
        end_time -> Timestamp,
        is_online -> Bool,
        charges -> Nullable<Float8>,
        currency -> Nullable<Varchar>,
        link -> Varchar,
        venue -> Nullable<Varchar>,
    }
}

table! {
    meetup_groups (id) {
        id -> Int4,
        slug -> Varchar,
        name -> Varchar,
        link -> Varchar,
        description -> Varchar,
        city -> Varchar,
        state -> Varchar,
        country -> Varchar,
        is_private -> Bool,
        photo -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(meetup_events, meetup_groups,);
