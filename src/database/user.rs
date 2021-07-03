use crate::util::dbhandler::*;

use sqlite::*;
use std::*;

pub struct User {
    pub id : i64,
    pub name : String,
    pub email : String,
}

/// Gets a user from database using it's id.
/// param : 
///
/// user_id - the user's id
/// 
/// Returns a User struct instance.
pub fn get_user(
    user_id : i64,
    db : &Database,
    ) -> Result<Option<User>> {

    let statement = db.connection.prepare("SELECT * FROM User WHERE id = :user_id;").unwrap();
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":user_id", Value::Integer(user_id.into()))])?;

    match cursor.next().unwrap() {
        Some(row) => {
            let user = User {
                id : row[0].as_integer().unwrap().to_owned(),
                name : row[1].as_string().unwrap().to_owned(),
                email : row[2].as_string().unwrap().to_owned(),

            };
            Ok(Some(user))
        }
        //
        // This is freaking weird. Trying to make proper error handling but it's
        // a freaking mess.
        None => Err(Error { 
            code : Some(1),
            message : Some(String::from("Something went wrong")),
        })
    }
}

fn get_user_from_credentials(
    user_name : &str,
    db : &Database,
    ) -> Option<User> {

    
    let statement = db.connection.prepare("SELECT * FROM User WHERE id = :user_id;").unwrap();
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":user_id", Value::String(user_name.into()))]);

    
    if let Some(row) = cursor.next().unwrap() {

        let user = User {
            id : row[0].as_integer().unwrap().to_owned(),
            name : row[1].as_string().unwrap().to_owned(),
            email : row[2].as_string().unwrap().to_owned(),

        };
        Some(user)
    }
    else {
        None
    }
}

