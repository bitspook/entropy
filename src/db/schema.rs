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
