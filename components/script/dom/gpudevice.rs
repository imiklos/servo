/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GPUDeviceBinding::{self, GPUDeviceMethods};
use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::{GPUExtensions, GPULimits};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::bindings::str::DOMString;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use webgpu::{wgpu, WebGPUDevice};
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
}

impl From<wgpu::Extensions> for GPUExtensions {
    fn from(extensions: wgpu::Extensions) -> Self {
        GPUExtensions {
            anisotropicFiltering: extensions.anisotropic_filtering,
        }
    }
}

impl From<wgpu::Limits> for GPULimits {
    fn from(limits: wgpu::Limits) -> Self {
        let mut lim = GPULimits::empty();
        lim.maxBindGroups = limits.max_bind_groups;
        lim
    }
}
