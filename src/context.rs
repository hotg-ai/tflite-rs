use std::ffi::CStr;
use std::fmt;

use bindings;

pub type ElementKind = bindings::TfLiteType;

pub trait ElemKindOf {
    fn elem_kind_of() -> ElementKind;
}

impl ElemKindOf for f32 {
    fn elem_kind_of() -> ElementKind {
        bindings::TfLiteType::kTfLiteFloat32
    }
}

impl ElemKindOf for u8 {
    fn elem_kind_of() -> ElementKind {
        bindings::TfLiteType::kTfLiteUInt8
    }
}

impl ElemKindOf for i32 {
    fn elem_kind_of() -> ElementKind {
        bindings::TfLiteType::kTfLiteInt32
    }
}

pub struct TensorInfo<'a> {
    pub(crate) handle: &'a bindings::TfLiteTensor,
}

impl<'a> fmt::Debug for TensorInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TensorInfo")
            .field("name", &self.name())
            .field("element_kind", &self.element_kind())
            .finish()
    }
}

impl<'a> TensorInfo<'a> {
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.handle.name) }
            .to_str()
            .unwrap()
    }

    pub fn element_kind(&self) -> ElementKind {
        self.handle.type_
    }
}
