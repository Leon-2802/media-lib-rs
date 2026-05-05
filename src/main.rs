mod database;

fn main() {
    use std::path::PathBuf;

    let db_path = PathBuf::from("media-lib.db");
    match database::Database::open(db_path.as_ref()) {
        Ok(db) => {
            println!("Database initialized at {:?}", db_path);
        }
        Err(e) => eprintln!("Failed to open database: {}", e),
    }
}
