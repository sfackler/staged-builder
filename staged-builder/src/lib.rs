//! A procedural macro which creates a "staged" builder for a type.
//!
//! Staged (also known as telescopic) builders are a style of infallible builders; code will not compile if any required
//! fields are not set. Specifically, the builder advances in order through a sequence of "stage" types, each
//! corresponding to a required field of the struct. The final stage has setters for all optional fields and the final
//! `.build()` method.
//!
//! See the documentation for [`#[staged_builder]`](staged_builder) for more details.
//!
//! # Examples
//!
//! ```
//! use staged_builder::staged_builder;
//!
//! #[staged_builder]
//! struct Person {
//!     #[builder(into)]
//!     name: String,
//!     age: u32,
//!     #[builder(list(item(type = Person)))]
//!     parents: Vec<Person>,
//! }
//!
//! # fn main() {
//! let person = Person::builder()
//!     .name("John Doe")
//!     .age(25)
//!     .build();
//! # }
//! ```
#![cfg_attr(not(doc), no_std)]

// Not part of the public API.
#[doc(hidden)]
pub use staged_builder_internals::__StagedBuilderInternalDerive;
#[doc(inline)]
pub use staged_builder_internals::staged_builder;

// Not part of the public API.
#[doc(hidden)]
pub mod __private {
    pub use core::convert::Into;
    pub use core::default::Default;
    pub use core::iter::{Extend, FromIterator, IntoIterator, Iterator};
    pub use core::result::Result;
}

/// A trait for types which validate their state before construction finishes.
///
/// The generated builder will call this method if the `#[builder(validate)]` attribute is placed at the struct level.
pub trait Validate {
    /// The error returned for an invalid value.
    type Error;

    /// Validates the state of `self`.
    fn validate(&self) -> Result<(), Self::Error>;
}

/// An example type using [`#[staged_builder]`](staged_builder).
#[cfg(doc)]
#[staged_builder]
#[builder(crate = crate)]
pub struct ExamplePerson {
    #[builder(into)]
    pub name: String,
    pub age: u32,
    #[builder(list(item(type = ExamplePerson)))]
    pub parents: Vec<ExamplePerson>,
}
