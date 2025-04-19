use super::*;

#[test]
fn test_is_safe() {
    let mut v = vec![1, 2, 3];
    assert_eq!(Ok(Rate::Increasing), check_vector(&v));

    v = vec![3, 2, 1];
    assert_eq!(Ok(Rate::Decreasing), check_vector(&v));

    v = vec![1, 1, 1];
    assert_eq!(Err(()), check_vector(&v));

    v = vec![1, 2, 2];
    assert_eq!(Err(()), check_vector(&v));

    v = vec![1, 4, 7];
    assert_eq!(Ok(Rate::Increasing), check_vector(&v));

    v = vec![7, 4, 1];
    assert_eq!(Ok(Rate::Decreasing), check_vector(&v));

    v = vec![1, 4, 8];
    assert_eq!(Err(()), check_vector(&v));

    v = vec![8, 5, 1];
    assert_eq!(Err(()), check_vector(&v));

    v = vec![1, 2, 1];
    assert_eq!(Err(()), check_vector(&v));
}

#[test]
fn test_subvec() {
    let v = vec![1, 2, 3];
    let e = vec![vec![2, 3], vec![1, 3], vec![1, 2]];
    let sv: Vec<Vec<i32>> = SubVec::new(v).collect();

    assert_eq!(e, sv);
}
