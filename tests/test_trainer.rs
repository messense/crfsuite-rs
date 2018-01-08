extern crate crfsuite;

use crfsuite::{Trainer, Algorithm, GraphicalModel, CrfError, Attribute};

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
fn test_trainer_train() {
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
}
