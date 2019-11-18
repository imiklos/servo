/* use crate::dom::bindings::codegen::Bindings::GPUBufferDescriptorBinding::GPUBufferDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUBufferUsageBinding::GPUBufferUsageConstants as constants;
use dom_struct::dom_struct;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::{self, GPUBufferSize, GPUBufferUsageFlags};

#[dom_struct]
pub struct GPUBufferDescriptor {
    reflector_ : Reflector,
    size: GPUBufferSize,//u64
    usage: GPUBufferUsageFlags,//u64
}

impl GPUBufferDescriptor {
    pub fn new_inherited() -> GPUBuffer {
        Self {
            reflector_: Reflector::new(),
            size: Default::default(),
            usage: Default::default(),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
    ) -> DomRoot<GPUBuffer> {
        reflect_dom_object(
            Box::new(GPUBufferDescriptor::new_inherited()),
            global,
            GPUBufferDescriptorBinding::Wrap,
        )
    }

    pub fn validate(&self) -> bool {
        //device lost? -> false
        match self.usage {
            constants::MAP_READ |
            constants::MAP_WRITE |
            constants::COPY_SRC |
            constants::COPY_DST |
            constants::INDEX |
            constants::VERTEX |
            constants::UNIFORM |
            constants::STORAGE |
            constants::INDIRECT => true,
            _ => false
        }
    }
} */