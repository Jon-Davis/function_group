# Function_Group

Function Group is a Function Overloading macro/hack for the rust programing language. The macro allows you to define multiple functions that take a variable number of arguments! *Actually the functions still only take one argument, but they accept multiple types of tuples*

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

## Known Bugs
  1. Refrences both mutable and immutable aren't bundled into the ty patern so while I look for a work around, refrences aren't supported

## Possible future features
  1. Generics should be feasble on a per sub-function basis
  2. having a function group in the impl block of a struct should be possible
  3. having a function group in a trait is unlikely
