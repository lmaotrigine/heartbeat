// make sure `std` is available but the prelude is not
#![no_std]
extern crate std;

use html::html;

#[test]
fn absolute_paths_in_generated() {
    let number = 42;
    let _ = html! { (number) };
}
