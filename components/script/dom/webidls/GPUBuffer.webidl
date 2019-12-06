/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpubuffer
[Exposed=(Window , DedicatedWorker), Serializable]
interface GPUBuffer {
    Promise<ArrayBuffer> mapReadAsync();
    Promise<ArrayBuffer> mapWriteAsync();
    void unmap();

    void destroy();
};
GPUBuffer includes GPUObjectBase;
//GPUBuffer includes GPUBufferUsage;

typedef unsigned long long GPUBufferSize;

typedef unsigned long GPUBufferUsageFlags;

typedef sequence<any> GPUMappedBuffer;