/// mod to make a command system with arguments/flags. This will be used to 
/// add worskpaces/collections and request, etc... 
///

use sqlite::*;
use crate::database::{container, user::User};
use crate::util::dbhandler::Database;

pub fn interpret_command(
    user : &User,
    db : &Database,
    name : &str,
    args : Vec<&str>,
    ) -> Result<()> {

    match name {
        "add" => {
            for (i, &arg) in args.iter().enumerate() {
                match arg {
                    "workspace" => {

                        let workspace = args[i+1];
                        container::create_workspace(user, workspace, db)?;

                    }

                    "collection" => {

                        let collection = args[i+1];
                        // container::create_collection(user, collection, db)?;

                    }

                    &_ => ()
                }
            }

        },
        "rm" => {

        },
        &_ => println!("command {} not found", name)

    }

    Ok(())

}

