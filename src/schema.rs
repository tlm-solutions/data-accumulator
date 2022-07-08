table! {
    stations (id) {
        id -> Uuid,
        token -> Nullable<VarChar>,
        name -> Text,
        lat -> Double,
        lon -> Double,
        region -> Text,
        owner -> Uuid,
        approved -> Bool,
    }
}
