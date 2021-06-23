table! {
    meetup_events (id) {
        id -> Text,
        group_id -> Text,
        created -> Timestamp,
        updated -> Timestamp,
        duration -> Nullable<Integer>,
        name -> Text,
        status -> Text,
        time -> Timestamp,
        local_date -> Text,
        local_time -> Text,
        utc_offset -> Integer,
        is_online_event -> Bool,
        link -> Text,
        description -> Nullable<Text>,
        how_to_find_us -> Nullable<Text>,
        visibility -> Nullable<Text>,
        member_pay_fee -> Nullable<Bool>,
        venue_visibility -> Nullable<Text>,
    }
}

table! {
    meetup_groups (id) {
        id -> Text,
        name -> Text,
        link -> Text,
        description -> Text,
        city -> Text,
        state -> Text,
        country -> Text,
        is_private -> Bool,
        member_count -> Integer,
        photo -> Nullable<Text>,
    }
}

joinable!(meetup_events -> meetup_groups (group_id));

allow_tables_to_appear_in_same_query!(
    meetup_events,
    meetup_groups,
);
