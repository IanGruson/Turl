/// mod to make a command system with arguments/flags. This will be used to 
/// add worskpaces/collections and request, etc... 
///

use sqlite::*;

pub fn interpret_command(
    name : &str,
    args : Vec<&str>,) -> Result<()> {

    match name {
        "add" => println!("Command add entered"),
        "rm" => println!("Command rm entered"),
        &_ => println!("Nothing entered")

    }

    Ok(())

}

