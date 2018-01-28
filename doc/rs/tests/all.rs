extern crate doc;

use std::path::Path;

#[test]
fn test_build_book() {
    let book_dir = Path::new(file!())
        .parent()
        .expect("directory of file")
        .join("book");
    assert_eq!(format!("{:?}", doc::build(book_dir)), "Ok(())");
}
