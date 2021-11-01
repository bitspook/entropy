table! {
    event_sections (name, event_id) {
        name -> Varchar,
        title -> Varchar,
        description -> Nullable<Varchar>,
        logo -> Nullable<Varchar>,
        desc_format -> Varchar,
        start_time -> Timestamp,
        end_time -> Timestamp,
        event_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    event_tags (event_id, tag_name) {
        event_id -> Uuid,
        tag_name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    events (id) {
        id -> Uuid,
        title -> Varchar,
        slug -> Varchar,
        description -> Nullable<Varchar>,
        desc_format -> Varchar,
        group_id -> Uuid,
        venue_id -> Nullable<Uuid>,
        photos -> Array<Text>,
        source -> Nullable<Varchar>,
        source_link -> Nullable<Varchar>,
        start_time -> Timestamp,
        end_time -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    group_tags (group_id, tag_name) {
        group_id -> Uuid,
        tag_name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    groups (id) {
        id -> Uuid,
        name -> Varchar,
        slug -> Varchar,
        description -> Nullable<Varchar>,
        desc_format -> Varchar,
        photos -> Array<Text>,
        source -> Nullable<Varchar>,
        source_link -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    organizations (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Varchar>,
        website -> Nullable<Varchar>,
        logo -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    tags (name) {
        name -> Varchar,
        display_name -> Nullable<Varchar>,
        icon -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    venues (id) {
        id -> Uuid,
        address -> Varchar,
        directions -> Nullable<Varchar>,
        organization_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(event_sections -> events (event_id));
joinable!(event_tags -> events (event_id));
joinable!(event_tags -> tags (tag_name));
joinable!(events -> groups (group_id));
joinable!(events -> venues (venue_id));
joinable!(group_tags -> groups (group_id));
joinable!(group_tags -> tags (tag_name));
joinable!(venues -> organizations (organization_id));

allow_tables_to_appear_in_same_query!(
    event_sections,
    event_tags,
    events,
    group_tags,
    groups,
    organizations,
    tags,
    venues,
);
