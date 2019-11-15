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
use wgpu::{adapter_get_info, adapter_request_device};
use wgpu::TypedId;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPUResponse {
    RequestAdapter(String, WebGPUAdapter),
    RequestDevice(WebGPUDevice, wgpu::DeviceDescriptor),
    MapReadAsync,
    MapWriteAsync,
}

pub type WebGPUResponseResult = Result<WebGPUResponse, String>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    RequestAdapter(IpcSender<WebGPUResponseResult>, wgpu::RequestAdapterOptions),
<<<<<<< HEAD
    RequestDevice,
    Exit(IpcSender<()>),
=======
    RequestDevice(
        IpcSender<WebGPUResponseResult>,
        WebGPUAdapter,
        wgpu::DeviceDescriptor,
    ),
    MapReadAsync,
    MapWriteAsync,
    Exit,
>>>>>>> 1a30d91f45... WebGPU impl
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
<<<<<<< HEAD
                WebGPURequest::RequestDevice => {},
                WebGPURequest::Exit(sender) => {
                    self.deinit();
                    if let Err(e) = sender.send(()) {
                        warn!("Failed to send response to WebGPURequest::Exit ({})", e)
                    }
                    return;
=======
                WebGPURequest::RequestDevice(sender, adapter , options) => {
                    let id = wgpu::Id::zip(0, 0, wgpu::Backend::Vulkan);
                    let _output =
                        gfx_select!(id => adapter_request_device(&self.global, adapter.0, &options, id));
                    let device = WebGPUDevice(id);

                    sender
                        .send(Ok(WebGPUResponse::RequestDevice(device, options)))
                        .expect("Failed to send response");
                },
                WebGPURequest::MapReadAsync => {},
                WebGPURequest::MapWriteAsync => {},
                WebGPURequest::Exit => {
                    self.deinit();
                    return
>>>>>>> 1a30d91f45... WebGPU impl
                },
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct WebGPUAdapter(pub wgpu::AdapterId);

<<<<<<< HEAD
impl MallocSizeOf for WebGPUAdapter {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}
=======
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct WebGPUDevice(pub wgpu::DeviceId);
>>>>>>> 1a30d91f45... WebGPU impl
