#[macro_use] extern crate function_group;

#[test]
fn test_function_group() {
    function_group! {
        fn add -> usize {
            (one : usize, two : usize) {
                one + two
            }
            (one : usize, two : usize, three: usize) {
                add((one, two)) + three
            }
        }
    }

    assert!(add((5, 5)) == 10);
    assert!(add((5, 5, 5)) == 15);
}

#[test]
fn test_function_group_mutability() {
    function_group! {
        fn add_10 -> usize {
            (mut one : usize) {
                one += 10;
                one
            }
        }
    }
    assert!(add_10(1) == 11);
}