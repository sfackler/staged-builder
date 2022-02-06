#![no_std]

#[doc(hidden)]
pub use staged_builder_internals::__StagedBuilderInternalDerive;
// Not part of the pulbic API.
#[doc(inline)]
pub use staged_builder_internals::staged_builder;

// Not part of the public API.
#[doc(hidden)]
pub mod __private {
    pub use core::convert::Into;
    pub use core::default::Default;
}
