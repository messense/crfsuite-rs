extern crate crfsuite;

use crfsuite::{Model, Attribute};

#[test]
fn test_open_model() {
    let model = Model::from_file("tests/model.crfsuite").unwrap();
    let tagger = model.tagger().unwrap();
    let _labels = tagger.labels().unwrap();
}

#[test]
fn test_open_not_existing_model_does_not_panic() {
    let ret = Model::from_file("tests/does-not-exists.crfsuite");
    assert!(ret.is_err());
}

#[test]
fn test_dump_model() {
    let model = Model::from_file("tests/model.crfsuite").unwrap();
    model.dump_file("tests/model.dump").unwrap();
}


#[test]
fn test_create_model_from_memory() {
    let model_memory = include_bytes!("model.crfsuite");
    let model = Model::from_memory(&model_memory[..]).unwrap();
    let mut tagger = model.tagger().unwrap();
    let xseq = vec![
        vec![Attribute::new("walk", 1.0), Attribute::new("shop", 0.5)],
        vec![Attribute::new("walk", 1.0)],
        vec![Attribute::new("walk", 1.0), Attribute::new("clean", 0.5)],
        vec![Attribute::new("shop", 0.5), Attribute::new("clean", 0.5)],
        vec![Attribute::new("walk", 0.5), Attribute::new("clean", 1.0)],
        vec![Attribute::new("clean", 1.0), Attribute::new("shop", 0.1)],
        vec![Attribute::new("walk", 1.0), Attribute::new("shop", 0.5)],
        vec![],
        vec![Attribute::new("clean", 1.0)],
    ];
    let yseq = ["sunny", "sunny", "sunny", "rainy", "rainy", "rainy", "sunny", "sunny", "rainy"];
    let res = tagger.tag(&xseq).unwrap();
    assert_eq!(res, yseq);

    tagger.probability(&yseq).unwrap();
    tagger.marginal("sunny", 1i32).unwrap();
}

#[test]
fn test_tag() {
    let model = Model::from_file("tests/model.crfsuite").unwrap();
    let mut tagger = model.tagger().unwrap();
    let xseq = vec![
        vec![Attribute::new("walk", 1.0), Attribute::new("shop", 0.5)],
        vec![Attribute::new("walk", 1.0)],
        vec![Attribute::new("walk", 1.0), Attribute::new("clean", 0.5)],
        vec![Attribute::new("shop", 0.5), Attribute::new("clean", 0.5)],
        vec![Attribute::new("walk", 0.5), Attribute::new("clean", 1.0)],
        vec![Attribute::new("clean", 1.0), Attribute::new("shop", 0.1)],
        vec![Attribute::new("walk", 1.0), Attribute::new("shop", 0.5)],
        vec![],
        vec![Attribute::new("clean", 1.0)],
    ];
    let yseq = ["sunny", "sunny", "sunny", "rainy", "rainy", "rainy", "sunny", "sunny", "rainy"];
    let res = tagger.tag(&xseq).unwrap();
    assert_eq!(res, yseq);

    tagger.probability(&yseq).unwrap();
    tagger.marginal("sunny", 1i32).unwrap();
}
