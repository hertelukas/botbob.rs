// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> BigInt,
        points -> BigInt,
        username -> Text,
    }
}
