/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUDeviceBinding::{self, GPUDeviceMethods};
use crate::dom::bindings::codegen::Bindings::GPUBufferDescriptorBinding::GPUBufferDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUBufferUsageBinding::GPUBufferUsageConstants as constants;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
//use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::{GPUExtensions, GPULimits};
use js::jsapi::JS_GetArrayBufferViewBuffer;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::gpubuffer::GPUBuffer;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::JSContext as SafeJSContext;
use ipc_channel::ipc;
use dom_struct::dom_struct;
use webgpu::{wgpu, WebGPUDevice, WebGPURequest, WebGPUResponse};
use js::jsapi::{Heap, JSObject};
use js::jsval::JSVal;
use js::jsval::UndefinedValue;
use js::jsval::ObjectValue;
use js::typedarray::{ArrayBuffer, CreateWith};
use std::ptr;
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
    device: WebGPUDevice,
}

impl GPUDevice {
    fn new_inherited(
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>,
        device: WebGPUDevice,
    ) -> GPUDevice {
        Self {
            eventtarget: EventTarget::new_inherited(),
            adapter: Dom::from_ref(adapter),
            extensions,
            limits,
            label: DomRefCell::new(None),
            device,
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>,
        device: WebGPUDevice,
    ) -> DomRoot<GPUDevice> {
        reflect_dom_object(
            Box::new(GPUDevice::new_inherited(
                adapter, extensions, limits, device,
            )),
            global,
            GPUDeviceBinding::Wrap,
        )
    }
}

impl GPUDeviceMethods for GPUDevice {
    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-adapter
    fn Adapter(&self) -> DomRoot<GPUAdapter> {
        DomRoot::from_ref(&self.adapter)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-extensions
    fn Extensions(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-limits
    fn Limits(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }

    fn CreateBuffer(&self, descriptor: &GPUBufferDescriptor) -> DomRoot<GPUBuffer> {
        let valid = match descriptor.usage {
            //constants::MAP_READ + constants::MAP_WRITE => false,
            constants::MAP_READ..=constants::INDIRECT => true,
            _ => false
        };
        let (sender, receiver) = ipc::channel().unwrap();
        if let Some(window) = self.global().downcast::<Window>() {
            let id = window.Navigator().create_buffer_id();
            match window.webgpu_channel() {
                Some(thread) => thread
                    .0
                    .send(WebGPURequest::CreateBuffer(sender, self.device, id))
                    .unwrap(),
                None => unimplemented!(),
            }
        } else {
            unimplemented!()
        };

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

        GPUBuffer::new(&self.global(), buffer, self.device/*, valid*/)
    }

    fn CreateBufferMapped(&self, cx: SafeJSContext, descriptor: &GPUBufferDescriptor) -> Vec<JSVal>{
        rooted!(in(*cx) let mut buffer = UndefinedValue());
        rooted!(in(*cx) let mut array_buffer = ptr::null_mut::<JSObject>());
            unsafe { ArrayBuffer::create(
                *cx,
                CreateWith::Slice(&array.as_slice()),
                array_buffer.handle_mut()
            )
            .is_ok() };
        let (sender, receiver) = ipc::channel().unwrap();
        if let Some(window) = self.global().downcast::<Window>() {
            let id = window.Navigator().create_buffer_id();
            match window.webgpu_channel() {
                Some(thread) => thread
                    .0
                    .send(WebGPURequest::CreateBufferMapped(sender, self.device, id, array_buffer.to_vec()))
                    .unwrap(),
                None => unimplemented!(),
            }
        } else {
            unimplemented!()
        };

        let (buffer, array) = match receiver.recv().unwrap() {
            Ok(resp) => {
                match resp {
                    WebGPUResponse::CreateBufferMapped(buffer, array) => (buffer, array),
                    _ => unimplemented!()
                }
            },
            Err(err) => unimplemented!(),
        };
        let buff = GPUBuffer::new(&self.global(), buffer, self.device/*, valid*/);
        let mut out = Vec::new();

        unsafe { buff.to_jsval(*cx, buffer.handle_mut()) };
        out.push(buffer.get());
        out.push(ObjectValue(array_buffer.get()));
        out
    }
}
