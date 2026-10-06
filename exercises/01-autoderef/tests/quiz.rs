use autoderef::quiz::*;
use autoderef::{Q1, Q2, Q3, Q4, Q5, Q6};

#[test]
fn q1_inherent_vs_trait() {
    assert_eq!(Q1, q1());
}

#[test]
fn q2_by_value_through_refs() {
    assert_eq!(Q2, q2());
}

#[test]
fn q3_deref_then_by_value() {
    assert_eq!(Q3, q3());
}

#[test]
fn q4_outer_trait_vs_inner_inherent() {
    assert_eq!(Q4, q4());
}

#[test]
fn q5_clone_without_bound() {
    assert_eq!(Q5, q5(&String::from("hi")));
}

#[test]
fn q6_clone_with_bound() {
    assert_eq!(Q6, q6(&String::from("hi")));
}
