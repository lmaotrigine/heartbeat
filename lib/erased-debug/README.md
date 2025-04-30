# erased-debug

This crate provides a wrapper type that prevents its debug representation from
being displayed when it is, for example, a field of a struct that derives
`Debug`.

Items declared as `Erased<T>` will only ever print the `Display` format of `T`,
(if it is implemented) when it is formatted with the alternate flag (`#`).

## See also

[`derivative`](https://lib.rs/crates/derivative): Gives you a lot more control
over your `derive` attributes, at the cost of extended compilation time of a
procedural macro.
