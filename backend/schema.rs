// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        address -> Varchar,
        username -> Varchar,
        position -> Nullable<Int8>,
        is_registered -> Bool,
        highest_score -> Nullable<Int8>,
        games_played -> Nullable<Int8>,
        updated -> Bool,
        registered_at -> Timestamp,
    }
}
