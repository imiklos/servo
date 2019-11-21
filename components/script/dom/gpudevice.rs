/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GPUDeviceBinding::{self, GPUDeviceMethods};
use crate::dom::bindings::codegen::Bindings::GPUBufferDescriptorBinding::GPUBufferDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::{GPUExtensions, GPULimits};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::bindings::str::DOMString;
use crate::dom::gpubuffer::GPUBuffer;
use crate::script_runtime::JSContext as SafeJSContext;
use ipc_channel::ipc;
use dom_struct::dom_struct;
use webgpu::{wgpu, WebGPUDevice, WebGPURequest, WebGPUResponse};
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;

#[dom_struct]
pub struct GPUDevice {
    eventtarget: EventTarget,
    adapter: Dom<GPUAdapter>,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    limits: Heap<*mut JSObject>,
    label: DomRefCell<Option<DOMString>>,
    #[ignore_malloc_size_of = "Arc"]
    device: WebGPUDevice,
}

impl GPUDevice {
    pub fn new_inherited(
        adapter: &GPUAdapter,
        /* extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject> , */
        device: WebGPUDevice,
    ) -> GPUDevice {
        Self {
            eventtarget: EventTarget::new_inherited(),
            adapter: Dom::from_ref(adapter),
            extensions: Heap::default(),
            limits: Heap::default(),
            label: DomRefCell::new(None),
            device
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        adapter: &GPUAdapter,
        descriptor: wgpu::DeviceDescriptor,
        /* extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>, */
        device: WebGPUDevice,
    ) -> DomRoot<GPUDevice> {
        reflect_dom_object(
            Box::new(GPUDevice::new_inherited(adapter, device)),
            global,
            GPUDeviceBinding::Wrap,
        )
    }
}

impl Drop for GPUDevice {
    fn drop(&mut self){
        println!("###DROPDevice");
    }
}

impl GPUDeviceMethods for GPUDevice {
    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-adapter
    fn Adapter(&self) -> DomRoot<GPUAdapter> {
        DomRoot::from_ref(&self.adapter)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-extensions
    fn Extensions(&self, cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-limits
    fn Limits(&self, cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }

    fn CreateBuffer(&self, descriptor: &GPUBufferDescriptor) -> DomRoot<GPUBuffer> {
        let _valid = match wgpu::BufferUsage::from_bits(descriptor.usage) {
            Some(usage) => true,
            None => false,
        };
        let desc = wgpu::BufferDescriptor {
            size: descriptor.size,
            usage: wgpu::BufferUsage::from_bits(descriptor.usage).unwrap(),
        };
        let (sender, receiver) = ipc::channel().unwrap();
        match self.global().as_window().webgpu_channel() {
            Some(thread) => {
                thread
                    .0
                    .send(WebGPURequest::CreateBuffer(sender, self.device, desc.clone()))
                    .unwrap()
            },
            None => {},
        }

        let buffer = match receiver.recv().unwrap() {
            Ok(resp) => {
                match resp {
                    WebGPUResponse::CreateBuffer(buffer) => buffer,
                    _ => unimplemented!()
                }
            },
            Err(err) => unimplemented!(),
        };
        std::dbg!(println!("BUFFER: {:?}", buffer));

        GPUBuffer::new(&self.global(), buffer, self.device, desc/*, valid*/)
    }

    fn CreateBufferMapped(&self, cx: SafeJSContext, descriptor: &GPUBufferDescriptor) -> Vec<JSVal> {
        let desc = wgpu::BufferDescriptor {
            size: descriptor.size,
            usage: wgpu::BufferUsage::from_bits(descriptor.usage).unwrap(),
        };
        let (sender, receiver) = ipc::channel().unwrap();
        match self.global().as_window().webgpu_channel() {
            Some(thread) => {
                thread
                    .0
                    .send(WebGPURequest::CreateMappedBuffer(sender, self.device, desc.clone()))
                    .unwrap()
            },
            None => {},
        }

        let buffer = match receiver.recv().unwrap() {
            Ok(resp) => {
                match resp {
                    WebGPUResponse::CreateMappedBuffer(buffer) => buffer,
                    _ => unimplemented!()
                }
            },
            Err(err) => unimplemented!(),
        };
        Vec::new(UndefinedValue())
    }
}

impl From<wgpu::Extensions> for GPUExtensions {
    fn from(extensions: wgpu::Extensions) -> Self {
        GPUExtensions {
            anisotropicFiltering: extensions.anisotropic_filtering,
        }
    }
}

/* impl From<u32> for wgpu::BufferUsage {
    fn from(usage: u32) -> Self {
        wgpu::BufferUsage {
            MAP_READ: usage << 32,
            MAP_WRITE: usage << 31,
            COPY_SRC: usage << 30,
            COPY_DST: usage << 29,
            INDEX: usage << 28,
            VERTEX: usage << 27,
            UNIFORM: usage << 26,
            STORAGE: usage << 25,
            STORAGE_READ: usage << 24,
            INDIRECT: usage << 23,
            NONE: usage << 22,
        }
    }
} */

impl From<wgpu::Limits> for GPULimits {
    fn from(limits: wgpu::Limits) -> Self {
        let mut lim = GPULimits::empty();
        lim.maxBindGroups = limits.max_bind_groups;
        lim
    }
}
