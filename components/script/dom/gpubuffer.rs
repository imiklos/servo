/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::{self, GPUBufferMethods, GPUBufferSize};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::globalscope::GlobalScope;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::reflector::DomObject;
use dom_struct::dom_struct;
use crate::dom::gpu::response_async;
use crate::dom::bindings::error::Error;
use crate::dom::promise::Promise;
use std::rc::Rc;
use crate::dom::gpu::AsyncWGPUListener;
use webgpu::{WebGPUResponse, WebGPUBuffer, WebGPURequest, WebGPUDevice};
use crate::compartments::InCompartment;

pub enum GPUBufferState {
    Mapped,
    Unmapped,
    Destroyed,
}

#[dom_struct]
pub struct GPUBuffer {
    reflector_ : Reflector,
    label: DomRefCell<Option<DOMString>>,
    size: GPUBufferSize,// u64
    //usage: GPUBufferUsage,
    #[ignore_malloc_size_of = "Arc"]
    state: DomRefCell<GPUBufferState>,
    #[ignore_malloc_size_of = "Arc"]
    buffer: WebGPUBuffer,
    #[ignore_malloc_size_of = "Arc"]
    device: WebGPUDevice,
    //valid: bool # Create invalid buffer...
}

impl GPUBuffer {
    pub fn new_inherited(
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
    ) -> GPUBuffer {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            size: Default::default(),
            state: DomRefCell::new(GPUBufferState::Unmapped),
            buffer,
            device,
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
    ) -> DomRoot<GPUBuffer> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(buffer, device)),
            global,
            GPUBufferBinding::Wrap,
        )
    }
}

impl GPUBufferMethods for GPUBuffer {
    fn MapReadAsync(&self, comp: InCompartment) -> Rc<Promise> {
        let promise = Promise::new_in_current_compartment(&self.global(), comp);
        let sender = response_async(&promise, self);

        match self.global().as_window().webgpu_channel() {
            Some(thread) => {
                thread
                    .0
                    .send(WebGPURequest::MapReadAsync(sender, self.buffer))
                    .unwrap()
            },
            None => promise.reject_error(Error::Type("No WebGPU thread...".to_owned())),
        }
        promise
    }

    fn MapWriteAsync(&self, comp: InCompartment) -> Rc<Promise> {
        let promise = Promise::new_in_current_compartment(&self.global(), comp);
        let sender = response_async(&promise, self);

        match self.global().as_window().webgpu_channel() {
            Some(thread) => {
                /* thread
                    .0
                    .send(WebGPURequest::MapWriteAsync())
                    .unwrap() */
            },
            None => promise.reject_error(Error::Type("No WebGPU thread...".to_owned())),
        }
        promise
    }

    fn Unmap(&self) {

    }

    fn Destroy(&self) {
        match *self.state.borrow() {
            GPUBufferState::Mapped => {},//unmap
            _ => {},
        };
        match self.global().as_window().webgpu_channel() {
            Some(thread) => {
                thread
                    .0
                    .send(WebGPURequest::DestroyBuffer(self.buffer))
                    .unwrap()
            },
            None => {},
        }
        *self.state.borrow_mut() = GPUBufferState::Destroyed;
    }

    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}

impl AsyncWGPUListener for GPUBuffer {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::MapReadAsync => {

            },
            WebGPUResponse::MapWriteAsync => {},
            _ => promise.reject_error(Error::Type(
                "Wrong response type from WebGPU thread...".to_owned(),
            )),
        }
    }
}
