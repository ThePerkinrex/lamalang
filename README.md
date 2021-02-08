This is just an exercise, but the language has basically the same syntax as Rust, and the objective is to write a parser, a type checker, and finally, multiple backends, including an interpreter and probably JS, and C.

Rust could be had as a backend, but the problem is the borrow checker, which would mean that it would be possible for the generated Rust not to compile because of semantics.

Check out the simple lamalib located in `src/lamalib`, as well as the examples in `examples`