extern crate xgboost;
extern crate ndarray;

use std::io::{BufRead, BufReader};
use std::fs::File;
use xgboost::{parameters, dmatrix::DMatrix, booster::Booster};

fn main() {
    // Load train and test matrices from text files (in LibSVM format).
    println!("Custom objective example...");
    let dtrain = DMatrix::load("../../xgboost-sys/xgboost/demo/data/agaricus.txt.train", false).unwrap();
    let dtest = DMatrix::load("../../xgboost-sys/xgboost/demo/data/agaricus.txt.test", false).unwrap();

    // Custom objective function
    fn log_reg_obj(preds: &[f32], dtrain: &DMatrix) -> (Vec<f32>, Vec<f32>) {
        let mut preds = ndarray::Array1::from_vec(preds.to_vec());
        preds.map_inplace(|x| *x = (-*x).exp());
        preds = 1.0 / (1.0 + preds);

        let labels: ndarray::Array1<f32> = ndarray::Array1::from_vec(dtrain.get_labels().unwrap().to_vec());
        let gradient = &preds - &labels;
        let hessian = &preds * &(1.0 - &preds);

        (gradient.to_vec(), hessian.to_vec())
    }

    // Configure objectives, metrics, etc.
    let learning_params = parameters::learning::LearningTaskParametersBuilder::default()
        .objective(parameters::learning::Objective::Custom(log_reg_obj))
        .build().unwrap();

    // Configure booster to use tree model, and configure tree parameters.
    let booster_params = parameters::booster::BoosterParameters::GbTree(
        parameters::tree::TreeBoosterParametersBuilder::default()
            .max_depth(2)
            .eta(1.0)
            .build().unwrap()
    );

    // Overall configuration for XGBoost.
    //
    // Note: for customised objective function, leave objectives as default.
    let params = parameters::ParametersBuilder::default()
        .learning_params(learning_params)
        .booster_params(booster_params)
        .silent(true)
        .build().unwrap();

    // Specify datasets to evaluate against during training.
    let evaluation_sets = [(&dtest, "test"), (&dtrain, "train")];

    // Number of boosting rounds to run during training.
    let num_round = 2;

    // Train booster model, and print evaluation metrics.
    println!("\nTraining tree booster...");
    let bst = Booster::train(&params, &dtrain, num_round, &evaluation_sets).unwrap();

    // Get predictions probabilities for given matrix (as ndarray::Array1).
    let preds = bst.predict(&dtest).unwrap();

    // Get predicted labels for each test example (i.e. 0 or 1).
    println!("\nChecking predictions...");
    let labels = dtest.get_labels().unwrap();
    println!("First 3 predicated labels: {} {} {}", labels[0], labels[1], labels[2]);

    // Print error rate.
    let num_correct: usize = preds.iter()
        .map(|&v| if v > 0.5 { 1 } else { 0 })
        .sum();
    println!("error={} ({}/{} correct)", num_correct as f32 / preds.len() as f32, num_correct, preds.len());
}
