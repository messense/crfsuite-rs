extern crate crfsuite;

use crfsuite::{Trainer, Algorithm, GraphicalModel};

#[test]
fn test_trainer_init() {
    let mut trainer = Trainer::new();
    trainer.init().unwrap();
    trainer.select(Algorithm::LBFGS, GraphicalModel::CRF1D).unwrap();
}
