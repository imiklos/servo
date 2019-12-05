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
use wgpu::{adapter_get_info, adapter_request_device, device_create_buffer, buffer_destroy};
use crate::wgpu::TypedId;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPUResponse {
    RequestAdapter(String, WebGPUAdapter),
    RequestDevice(WebGPUDevice, wgpu::DeviceDescriptor),
    CreateBuffer(WebGPUBuffer),
    MapReadAsync,
    MapWriteAsync,
}

pub type WebGPUResponseResult = Result<WebGPUResponse, String>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    RequestAdapter(
        IpcSender<WebGPUResponseResult>,
        wgpu::RequestAdapterOptions,
        wgpu::AdapterId,
    ),
    RequestDevice(
        IpcSender<WebGPUResponseResult>,
        WebGPUAdapter,
        wgpu::DeviceDescriptor,
        wgpu::DeviceId,
    ),
    Exit(IpcSender<()>),
    CreateBuffer(IpcSender<WebGPUResponseResult>, WebGPUDevice),
    DestroyBuffer(WebGPUBuffer),
    MapReadAsync,
    MapWriteAsync,
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
}

impl WGPU {
    fn new(receiver: IpcReceiver<WebGPURequest>) -> Self {
        WGPU {
            receiver,
            global: wgpu::Global::new("webgpu-native"),
            adapters: Vec::new(),
            _invalid_adapters: Vec::new(),
        }
    }

    fn deinit(self) {
        self.global.delete()
    }

    fn run(mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                WebGPURequest::RequestAdapter(sender, options, id) => {
                    let adapter_id = match wgpu::request_adapter(&self.global, &options, &[id]) {
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
                WebGPURequest::RequestDevice(sender, adapter, descriptor, id) => {
                    let _output = gfx_select!(id => adapter_request_device(&self.global, adapter.0, &descriptor, id));
                    let device = WebGPUDevice(id);
                    if let Err(e) =
                        sender.send(Ok(WebGPUResponse::RequestDevice(device, descriptor)))
                    {
                        warn!(
                            "Failed to send response to WebGPURequest::RequestDevice ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateBuffer(sender, device) => {
                    let id = wgpu::Id::zip(0, 0, wgpu::Backend::Vulkan);
                    let desc = wgpu::BufferDescriptor {
                        size: 16,
                        usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
                    };
                    let _output =
                        gfx_select!(id => device_create_buffer(&self.global, device.0, &desc, id));
                    let buffer = WebGPUBuffer(id);

                    sender
                        .send(Ok(WebGPUResponse::CreateBuffer(buffer)))
                        .expect("Failed to send response");
                },
                WebGPURequest::DestroyBuffer(buffer) => {
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

macro_rules! webgpu_resource {
    ($name:ident, $id:ty) => {
        #[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
        pub struct $name(pub $id);

        impl MallocSizeOf for $name {
            fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
                0
            }
        }

        impl Eq for $name {}
    };
}

webgpu_resource!(WebGPUAdapter, wgpu::AdapterId);
webgpu_resource!(WebGPUDevice, wgpu::DeviceId);
webgpu_resource!(WebGPUBuffer, wgpu::BufferId);
