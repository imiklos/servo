/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::InCompartment;
use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::{self, GPUAdapterMethods, GPUDeviceDescriptor, GPULimits, GPUExtensions};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::response_async;
use crate::dom::gpu::AsyncWGPUListener;
use crate::dom::gpudevice::GPUDevice;
use crate::dom::promise::Promise;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;
use std::rc::Rc;
use webgpu::{wgpu, WebGPUAdapter, WebGPURequest, WebGPUResponse};


#[dom_struct]
pub struct GPUAdapter {
    reflector_: Reflector,
    name: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    adapter: WebGPUAdapter,
}

impl GPUAdapter {
    pub fn new_inherited(
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        adapter: WebGPUAdapter,
    ) -> GPUAdapter {
        GPUAdapter {
            reflector_: Reflector::new(),
            name,
            extensions,
            adapter,
        }
    }

    pub fn new(
        global: &GlobalScope,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        adapter: WebGPUAdapter,
    ) -> DomRoot<GPUAdapter> {
        reflect_dom_object(
            Box::new(GPUAdapter::new_inherited(name, extensions, adapter)),
            global,
            GPUAdapterBinding::Wrap,
        )
    }
}

impl Drop for GPUAdapter {
    fn drop(&mut self) {
        println!("###DROPAdapter");
    }
}

impl GPUAdapterMethods for GPUAdapter {
    // https://gpuweb.github.io/gpuweb/#dom-gpuadapter-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://gpuweb.github.io/gpuweb/#dom-gpuadapter-extensions
    fn Extensions(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice
    fn RequestDevice(&self, descriptor: &GPUDeviceDescriptor, comp: InCompartment) -> Rc<Promise> {
        let promise = Promise::new_in_current_compartment(&self.global(), comp);
        let sender = response_async(&promise, self);

        match self.global().as_window().webgpu_channel() {
            Some(thread) => {
                thread
                    .0
                    .send(WebGPURequest::RequestDevice(
                        sender,
                        self.adapter,
                        descriptor.into(),
                    ))
                    .unwrap()
            },
            None => promise.reject_error(Error::Type("No WebGPU thread...".to_owned())),
        }
        promise
    }
}

impl From<&GPULimits> for wgpu::Limits {
    fn from(limits: &GPULimits) -> Self {
        wgpu::Limits {
            max_bind_groups: limits.maxBindGroups,
        }
    }
}

impl From<&GPUExtensions> for wgpu::Extensions {
    fn from(extensions: &GPUExtensions) -> Self {
        wgpu::Extensions {
            anisotropic_filtering: extensions.anisotropicFiltering,
        }
    }
}

impl From<&GPUDeviceDescriptor> for wgpu::DeviceDescriptor {
    fn from(descriptor: &GPUDeviceDescriptor) -> Self {
        wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions::from(&descriptor.extensions),
            limits: wgpu::Limits::from(&descriptor.limits),
        }
    }
}

impl AsyncWGPUListener for GPUAdapter {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::RequestDevice(deviceID, options) => {
                let device =
                    GPUDevice::new(&self.global(), &self, options, deviceID);
                promise.resolve_native(&device);
            },
            _ => promise.reject_error(Error::Type(
                "Wrong response type from WebGPU thread...".to_owned(),
            )),
        }
    }
}
