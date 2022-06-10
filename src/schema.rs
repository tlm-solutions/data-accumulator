/*table! {
    users (id) {
        id -> Uuid,
        name -> Text,
        email -> Text,
        password -> VarChar,
        role -> Integer // maybe enum
    }
}
table! {
    region (id) {
        id -> Integer,
        name -> Text,
        transport_company -> Text,
        frequency -> Integer,
        protocol -> Text
    }
}
*/
table!{
    stations (id) {
        id -> Uuid,
        token -> Nullable<VarChar>,
        name -> Text,
        lat -> Double,
        lon -> Double,
        region -> Integer,
        owner -> Uuid,
        approved -> Bool,
    }
}

