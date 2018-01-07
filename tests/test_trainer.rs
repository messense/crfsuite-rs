extern crate crfsuite;

use crfsuite::{Trainer, Algorithm, GraphicalModel, CrfError};

#[test]
fn test_trainer_train_uninitialized() {
    let mut trainer = Trainer::new();
    let ret = trainer.train("test.crfsuite", 1i32);
    assert_eq!(ret.err(), Some(CrfError::AlgorithmNotSelected));
}

#[test]
fn test_trainer_train_empty_data() {
    let mut trainer = Trainer::new();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
    let ret = trainer.train("test.crfsuite", 1i32);
    assert_eq!(ret.err(), Some(CrfError::EmptyData));
}
