table! {
    meetup_events (id) {
        id -> Text,
        group_slug -> Text,
        title -> Text,
        description -> Nullable<Text>,
        start_time -> Timestamp,
        end_time -> Timestamp,
        is_online -> Bool,
        charges -> Nullable<Double>,
        currency -> Nullable<Text>,
        link -> Text,
        venue -> Nullable<Text>,
    }
}

table! {
    meetup_groups (id) {
        id -> Text,
        slug -> Text,
        name -> Text,
        link -> Text,
        description -> Text,
        city -> Text,
        state -> Text,
        country -> Text,
        is_private -> Bool,
        photo -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    meetup_events,
    meetup_groups,
);
