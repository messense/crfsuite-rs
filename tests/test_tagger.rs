extern crate crfsuite;

use crfsuite::Model;

#[test]
fn test_open_model() {
    let model = Model::from_file("tests/model.crfsuite").unwrap();
    let tagger = model.tagger().unwrap();
    let labels = tagger.labels().unwrap();
    println!("{:?}", labels);
}
