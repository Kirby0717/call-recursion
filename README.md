# call-recursion

Do recursion on the heap
===

This crate provides a method to avoid stack overflows
by converting async functions into state machines and
doing recursion on the heap.

## Usage

``` rust
// Import trait
use call_recursion::FutureRecursion;

// Writing deeply recursive functions async
async fn pow_mod(base: usize, n: usize, r#mod: usize) -> usize {
    if n == 0 {
        1
    }
    else {
        // Call 'recurse' method to recurse over the heap
        // 'recurse' return Future
        (base * pow_mod(base, n - 1, r#mod).recurse().await) % r#mod
    }
}

fn main() {
    // Call 'start_recursion' method at the beginning of the recursion.
    // Return value of 'start_recursion' is not changed
    println!("{}", pow_mod(2, 10_000_000, 1_000_000).start_recursion());
}
```

## License
Licensed under either of Apache License, Version 2.0 or MIT license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
