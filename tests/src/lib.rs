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
        fn add_too {
            (one : &mut usize, two : usize) {
                *one += two;
            }
        }
    }

    let mut x : usize = 10;
    add_too((&mut x, 5 as usize));
    assert!(x == 15);
}