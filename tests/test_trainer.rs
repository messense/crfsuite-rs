extern crate crfsuite;

use crfsuite::{Trainer, Algorithm, GraphicalModel, CrfError, Attribute, Model};

#[test]
fn test_trainer_train_uninitialized() {
    let mut trainer = Trainer::new();
    let ret = trainer.train("tests/test.crfsuite", 1i32);
    assert_eq!(ret.err(), Some(CrfError::AlgorithmNotSelected));
}

#[test]
fn test_trainer_train_empty_data() {
    let mut trainer = Trainer::new();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
    let ret = trainer.train("tests/test.crfsuite", -1i32);
    assert_eq!(ret.err(), Some(CrfError::EmptyData));
}

#[test]
fn test_train_and_tag() {
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
    let mut trainer = Trainer::new();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
    trainer.append(&xseq, &yseq, 0i32).unwrap();
    trainer.train("tests/test.crfsuite", -1i32).unwrap();
    drop(trainer);

    // tag
    let model = Model::from_file("tests/test.crfsuite").unwrap();
    let mut tagger = model.tagger().unwrap();
    let res = tagger.tag(&xseq).unwrap();
    assert_eq!(res, yseq);
}

#[test]
fn test_clear_empty() {
    let mut trainer = Trainer::new();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
    trainer.clear().unwrap();
}

#[test]
fn test_clear_not_empty() {
    let xseq = vec![
        vec![Attribute::new("walk", 1.0), Attribute::new("shop", 0.5)],
    ];
    let yseq = ["sunny"];
    let mut trainer = Trainer::new();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
    trainer.append(&xseq, &yseq, 0i32).unwrap();
    trainer.clear().unwrap();
}

#[test]
fn test_params() {
    let mut trainer = Trainer::new();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
    let params = trainer.params();
    assert!(params.contains(&"c1".to_string()));
    assert!(params.contains(&"c2".to_string()));
    assert!(params.contains(&"num_memories".to_string()));

    trainer.select(Algorithm::L2SGD, GraphicalModel::CRF1D).unwrap();
    let params = trainer.params();
    assert!(!params.contains(&"c1".to_string()));
    assert!(params.contains(&"c2".to_string()));
}

#[test]
fn test_help() {
    let mut trainer = Trainer::new();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
    let msg = trainer.help("c1").unwrap();
    assert!(msg.contains("L1"));

    trainer.select(Algorithm::L2SGD, GraphicalModel::CRF1D).unwrap();
    let msg = trainer.help("c2").unwrap();
    assert!(msg.contains("L2"));
}

#[test]
fn test_help_invalid_argument() {
    let mut trainer = Trainer::new();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
    let ret = trainer.help("foo");
    match ret.err().unwrap() {
        CrfError::ParamNotFound(_) => {},
        _ => panic!("test fail")
    }
}
