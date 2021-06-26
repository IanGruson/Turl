use sqlite::*;

pub struct Database {
    pub filename : String,
    pub connection : Connection,
}

fn connect(mut database : Database) {
    database.connection = sqlite::open(":memory:").unwrap();
}
