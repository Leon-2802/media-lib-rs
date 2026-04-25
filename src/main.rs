mod database;

use database::models::{Test, ParentTest};

fn main() {
    let x = Test::new(42);
    let y = ParentTest::new(x.clone());
    println!("{x:?}");
    println!("{y:?}");
}
