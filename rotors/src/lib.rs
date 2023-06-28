pub mod codec;

pub use rotors_macros::rotors;
pub use tonic::async_trait;

#[doc(hidden)]
pub mod codegen {
    pub use tonic;
}
