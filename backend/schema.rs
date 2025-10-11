// @generated automatically by Diesel CLI.

diesel::table! {
    game_scores (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        wallet_address -> Varchar,
        score -> Int4,
        game_duration -> Int4,
        played_at -> Timestamp,
    }
}

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

diesel::joinable!(game_scores -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    game_scores,
    users,
);
