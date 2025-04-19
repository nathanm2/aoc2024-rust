use super::*;

#[test]
fn test_collapse_vec() {
    let v = vec![1, 2, 3, 3, 3, 3, 4, 9, 10, 10];
    let expected = HashMap::from([(1, 1), (2, 1), (3, 4), (4, 1), (9, 1), (10, 2)]);
    let result = collapse_vec(&v);
    assert_eq!(expected, result);
}
