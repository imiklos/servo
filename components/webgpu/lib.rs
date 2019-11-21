/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
pub extern crate wgpu_native as wgpu;

use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use servo_config::pref;
use std::collections::HashMap;
use wgpu::{adapter_get_info, adapter_request_device, device_create_buffer, buffer_destroy, device_destroy, device_poll};
use wgpu::TypedId;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPUResponse {
    RequestAdapter(String, WebGPUAdapter),
    RequestDevice(WebGPUDevice, wgpu::DeviceDescriptor),
    CreateBuffer(WebGPUBuffer),
    CreateMappedBuffer(GPUMappedBuffer),
    MapReadAsync,
    MapWriteAsync,
}

pub type WebGPUResponseResult = Result<WebGPUResponse, String>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    RequestAdapter(IpcSender<WebGPUResponseResult>, wgpu::RequestAdapterOptions),
    RequestDevice(
        IpcSender<WebGPUResponseResult>,
        WebGPUAdapter,
        wgpu::DeviceDescriptor,
    ),
    CreateBuffer(IpcSender<WebGPUResponseResult>, WebGPUDevice, wgpu::BufferDescriptor),
    CreateMappedBuffer(IpcSender<WebGPUResponseResult>, WebGPUDevice, wgpu::BufferDescriptor),
    DestroyBuffer(WebGPUBuffer),
    MapReadAsync,
    MapWriteAsync,
    Exit(IpcSender<()>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGPU(pub IpcSender<WebGPURequest>);

impl WebGPU {
    pub fn new() -> Option<Self> {
        if !pref!(dom.webgpu.enabled) {
            return None;
        }
        let (sender, receiver) = match ipc::channel() {
            Ok(sender_and_receiver) => sender_and_receiver,
            Err(e) => {
                warn!(
                    "Failed to create sender and receiciver for WGPU thread ({})",
                    e
                );
                return None;
            },
        };

        if let Err(e) = std::thread::Builder::new()
            .name("WGPU".to_owned())
            .spawn(move || {
                WGPU::new(receiver).run();
            })
        {
            warn!("Failed to spwan WGPU thread ({})", e);
            return None;
        }
        Some(WebGPU(sender))
    }

    pub fn exit(&self, sender: IpcSender<()>) -> Result<(), &'static str> {
        self.0
            .send(WebGPURequest::Exit(sender))
            .map_err(|_| "Failed to send Exit message")
    }
}

struct WGPU {
    receiver: IpcReceiver<WebGPURequest>,
    global: wgpu::Global,
    adapters: Vec<WebGPUAdapter>,
    // Track invalid adapters https://gpuweb.github.io/gpuweb/#invalid
    _invalid_adapters: Vec<WebGPUAdapter>,
    devices_adapters: HashMap<WebGPUDevice, WebGPUAdapter>,
    buffer_devices: HashMap<WebGPUBuffer, WebGPUDevice>,
}

impl WGPU {
    fn new(receiver: IpcReceiver<WebGPURequest>) -> Self {
        WGPU {
            receiver,
            global: wgpu::Global::new("webgpu-native"),
            adapters: Vec::new(),
            _invalid_adapters: Vec::new(),
            devices_adapters: HashMap::new(),
            buffer_devices: HashMap::new(),
        }
    }

    fn deinit(mut self) {
        for (buffer, device) in self.buffer_devices.drain() {
            std::dbg!(println!("#DEINIT: Buffer {:?}", buffer.0));
            gfx_select!(device.0 => device_poll(&self.global, device.0, true));
            let _out = gfx_select!(buffer.0 => buffer_destroy(&self.global, buffer.0));
            std::dbg!(println!("#DEINIT: Buffer {:?}", _out));
        }
        assert!(self.buffer_devices.is_empty());
        for (device, _adapter) in self.devices_adapters.drain() {
            std::dbg!(println!("#DEINIT: Device {:?}", device.0));
            let _out = gfx_select!(device.0 => device_destroy(&self.global, device.0));
            std::dbg!(println!("#DEINIT: Device {:?}", _out));
        }
        assert!(self.devices_adapters.is_empty());
        self.global.delete();
    }

    fn run(mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                WebGPURequest::RequestAdapter(sender, options) => {
                    let adapter_id = match wgpu::request_adapter(
                        &self.global,
                        &options,
                        &[
                            wgpu::Id::zip(0, 0, wgpu::Backend::Vulkan),
                            wgpu::Id::zip(0, 0, wgpu::Backend::Metal),
                            wgpu::Id::zip(0, 0, wgpu::Backend::Dx12),
                        ],
                    ) {
                        Some(id) => id,
                        None => {
                            if let Err(e) =
                                sender.send(Err("Failed to get webgpu adapter".to_string()))
                            {
                                warn!(
                                    "Failed to send response to WebGPURequest::RequestAdapter ({})",
                                    e
                                )
                            }
                            return;
                        },
                    };
                    let adapter = WebGPUAdapter(adapter_id);
                    self.adapters.push(adapter);
                    let info =
                        gfx_select!(adapter_id => adapter_get_info(&self.global, adapter_id));
                    if let Err(e) =
                        sender.send(Ok(WebGPUResponse::RequestAdapter(info.name, adapter)))
                    {
                        warn!(
                            "Failed to send response to WebGPURequest::RequestAdapter ({})",
                            e
                        )
                    }
                },
                WebGPURequest::RequestDevice(sender, adapter , options) => {
                    //generate ID
                    let id = wgpu::Id::zip(1, 0, wgpu::Backend::Vulkan);
                    let _output =
                    gfx_select!(id => adapter_request_device(&self.global, adapter.0, &options, id));
                    let device = WebGPUDevice(id);
                    self.devices_adapters.insert(device, adapter);
                    sender
                        .send(Ok(WebGPUResponse::RequestDevice(device, options)))
                        .expect("Failed to send response");
                },
                WebGPURequest::CreateBuffer(sender, device, descriptor) => {
                    let id = wgpu::Id::zip(2, 0, wgpu::Backend::Vulkan);
                    let _output =
                    gfx_select!(id => device_create_buffer(&self.global, device.0, &descriptor, id));
                    let buffer = WebGPUBuffer(id);
                    self.buffer_devices.insert(buffer, device);
                    sender
                    .send(Ok(WebGPUResponse::CreateBuffer(buffer)))
                    .expect("Failed to send response");
                },
                WebGPURequest::DestroyBuffer(buffer) => {
                    self.buffer_devices.remove(&buffer);
                    let _output =
                        gfx_select!(buffer.0 => buffer_destroy(&self.global, buffer.0));
                },
                WebGPURequest::MapReadAsync => {},
                WebGPURequest::MapWriteAsync => {},
                WebGPURequest::Exit(sender) => {
                    self.deinit();
                    if let Err(e) = sender.send(()) {
                        warn!("Failed to send response to WebGPURequest::Exit ({})", e)
                    }
                    return;
                },
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct WebGPUAdapter(pub wgpu::AdapterId);
impl MallocSizeOf for WebGPUAdapter {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct WebGPUDevice(pub wgpu::DeviceId);

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct WebGPUBuffer(pub wgpu::BufferId);
