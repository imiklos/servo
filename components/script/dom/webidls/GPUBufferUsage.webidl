[Exposed=(Window, DedicatedWorker)]
interface GPUBufferUsage {
    const GPUBufferUsageFlags MAP_READ  = 0x0001;
    const GPUBufferUsageFlags MAP_WRITE = 0x0002;
    const GPUBufferUsageFlags COPY_SRC  = 0x0004;
    const GPUBufferUsageFlags COPY_DST  = 0x0008;
    const GPUBufferUsageFlags INDEX     = 0x0010;
    const GPUBufferUsageFlags VERTEX    = 0x0020;
    const GPUBufferUsageFlags UNIFORM   = 0x0040;
    const GPUBufferUsageFlags STORAGE   = 0x0080;
    const GPUBufferUsageFlags INDIRECT  = 0x0100;
};