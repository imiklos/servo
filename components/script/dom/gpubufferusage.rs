use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use dom_struct::dom_struct;

#[dom_struct]
pub struct GPUBufferUsage {
    reflector_ : Reflector,
}