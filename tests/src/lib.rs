#[macro_use] extern crate function_group;

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

function_group! {
    fn add_to {
        (one : &mut usize, two : usize) {
            *one += two;
        }
        (one : &mut usize, two : usize, three : usize) {
            *one += two + three;
        }
    }
}

function_group! {
    fn test_mut -> usize {
        (mut one : usize, two : usize) {
            one += two;
            one
        }
    }
}

#[allow(dead_code)]
struct TestStruct(usize);

function_group! {
    fn add_to_struct(&mut self : TestStruct) {
        (one : usize) {
            self.0 += one;
        }
        (one : usize, two : usize){
            self.0 += add((one, two)); 
        }
    }
}

#[test]
fn test_function_group() {
    use crate::add;
    assert!(add((5, 5)) == 10);
    assert!(add((5, 5, 5)) == 15);
}


#[test]
fn test_function_group_mutability() {
    use crate::add_to;
    let mut x = 10;
    add_to((&mut x, 5));
    add_to((&mut x, 5, 5));
    assert!(x == 25);
    assert!(test_mut((x, 5)) == 30);
}

#[test]
fn test_function_group_method() {
    use crate::TestStruct;
    let mut x = TestStruct(10);
    x.add_to_struct((1,2));
    assert!(x.0 == 13);
}