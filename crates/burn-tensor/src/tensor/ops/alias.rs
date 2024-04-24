use crate::backend::{Backend, BackendBridge};

// We provide some type aliases to improve the readability of using associated types without
// having to use the disambiguation syntax.

/// Device type used by the backend.
pub type Device<B> = <B as Backend>::Device;

/// Float element type used by backend.
pub type FloatElem<B> = <B as Backend>::FloatElem;
/// Integer element type used by backend.
pub type IntElem<B> = <B as Backend>::IntElem;
/// Full precision float element type used by the backend.
pub type FullPrecisionBackend<B> =
    <<B as Backend>::FullPrecisionBridge as BackendBridge<B>>::Target;

/// Float tensor primitive type used by the backend.
pub type FloatTensor<B, const D: usize> = <B as Backend>::FloatTensorPrimitive<D>;
/// Integer tensor primitive type used by the backend.
pub type IntTensor<B, const D: usize> = <B as Backend>::IntTensorPrimitive<D>;
/// Boolean tensor primitive type used by the backend.
pub type BoolTensor<B, const D: usize> = <B as Backend>::BoolTensorPrimitive<D>;
