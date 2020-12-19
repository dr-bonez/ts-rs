#![allow(dead_code)]

use serde::Serialize;
use ts_rs::{export, TS};

#[derive(Serialize, TS)]
#[ts(rename_all = "lowercase")]
enum Role {
    User,
    #[ts(rename = "administrator")]
    Admin,
}

#[derive(Serialize, TS)]
// when 'serde-compat' is enabled, ts-rs tries to use supported serde attributes.
#[serde(rename_all = "UPPERCASE")]
enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Serialize, TS)]
struct User {
    user_id: i32,
    first_name: String,
    last_name: String,
    role: Role,
    #[ts(inline)]
    gender: Gender,
}

// this can be generated by the `export!` macro, see below
#[cfg(test)]
mod export_ts {
    use crate::{Role, User};
    use ts_rs::TS;

    #[test]
    fn export_ts() {
        let _ = std::fs::remove_file("bindings.ts");
        Role::dump("bindings.ts").unwrap();
        User::dump("bindings.ts").unwrap();
    }
}

#[derive(TS)] struct A;
#[derive(TS)] struct B((A, u8));



// this will export [Role] to `role.ts` and [User] to `user.ts`.
// `export!` will also take care of including imports in typescript files.
export! {
    Role => "role.ts",
    User => "user.ts",
    A => "a.ts",
    B => "b.ts"
}
