[![Crates.io](https://img.shields.io/crates/v/function_group.svg)](https://crates.io/crates/function_group)
[![Documentation](https://docs.rs/function_group/badge.svg)](https://docs.rs/function_group)
# Function_Group

Function Group is a Function Overloading macro for the rust programing language. The macro allows you to define multiple functions that take a variable number of arguments! *Actually the functions still only take one argument, but they accept multiple types of tuples*.

Function groups can take multiple types of arguments and even be recursive.
```rust
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
```

The arguments can be mutable or immutable refrences.
```rust
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

let mut x = 10;
add_to((&mut x, 5));
add_to((&mut x, 5, 5));
assert!(x == 25);
```

Function Groups can even be associated with a Type. In the example below, each sub function will be passed a mutable refrence to self, and these functions will be usable by the TestStruct type.
```rust
struct TestStruct(usize);
function_group! {
    fn add_to_struct(&mut self : TestStruct) {
        (one : usize) {
            self.0 += one;
        }
        (one : usize, two : usize){
            self.0 += one + two; 
        }
    }
}

let mut x = TestStruct(10);
x.add_to_struct((1,2));
assert!(x.0 == 13);
```

## Possible future features
  1. Generics should be feasble on a per sub-function basis
  2. having a function group in a trait is unlikely
