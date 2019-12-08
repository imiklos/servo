console.log('Compute Demo Starting...')
"use strict";
if (navigator.gpu) {
  document.getElementById('not-supported').style.display = 'none';
}

let matrixDimension = 1024;
let matrixElements = matrixDimension * matrixDimension;

// Not on the slides. Local size in X and Y. Without this the GPU will only run
// one instance of the compute shader on a block of (for example) 32 ALUs,
// wasting 31 of them.
const localSize = 8;

function yieldToBrowser() {
  return new Promise(function(resolve, reject) {
      setTimeout(function() {
          resolve();
      }, 0);
  });
}

async function setStatus(message) {
  document.getElementById('status').textContent = message;
  await yieldToBrowser();
}

async function computeOnGPU(matrixA, matrixB) {
  //console.log('import glslang..')
  const glslangModule = await import('./glsout');
  //const glslang = await glslangModule.default();
  //console.log('import glslang....DONE')

  await setStatus("Preparing for the GPU");

  // Slide 1: Initialize WebGPU
  await setStatus("Request Adapter");
  console.log('Requesting Adapter..')
  const adapter = await navigator.gpu.requestAdapter();
  console.log(adapter)
  await setStatus("Request Device");
  console.log('Requesting Device..')
  const device = await adapter.requestDevice();
  console.log(device)
  
  // Slide 2: Allocate memory for the matrix data.
  const matrixSize = matrixDimension * matrixDimension * 4; // sizeof(float) == 4
  await setStatus("Create Mapped Buffer");
  console.log('Create Buffer Mapped')
  console.log('GPUBufferUsage')
  console.log(GPUBufferUsage)
  for (const p in GPUBufferUsage) {
      console.log(p)
  }
  console.log(device.createBufferMapped)
  
  const [gpuMatrixA, cpuMatrixA] = device.createBufferMapped({
      size: matrixSize,
      usage: GPUBufferUsage.STORAGE,
  });

  console.log(gpuMatrixA)
  for (const p in gpuMatrixA) {
    console.log(p)
  }


  new Float32Array(cpuMatrixA).set(matrixA);
  gpuMatrixA.unmap();

  const [gpuMatrixB, cpuMatrixB] = device.createBufferMapped({
      size: matrixSize,
      usage: GPUBufferUsage.STORAGE,
  });

  new Float32Array(cpuMatrixB).set(matrixB);
  gpuMatrixB.unmap();

  const gpuMatrixC = device.createBuffer({
      size: matrixSize,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
  });

  // Layout things that are hidden
  const bindGroupLayout = device.createBindGroupLayout({
      bindings: [
          {binding: 0, visibility: GPUShaderStage.COMPUTE, type: "storage-buffer"},
          {binding: 1, visibility: GPUShaderStage.COMPUTE, type: "storage-buffer"},
          {binding: 2, visibility: GPUShaderStage.COMPUTE, type: "storage-buffer"},
      ],
  });
  const pipelineLayout = device.createPipelineLayout({
      bindGroupLayouts: [bindGroupLayout],
  });

  // Slide 3: Create the data “group”.
  const bindGroup = device.createBindGroup({
      layout: bindGroupLayout,
      bindings: [
          {binding: 0, resource: {buffer: gpuMatrixA}},
          {binding: 1, resource: {buffer: gpuMatrixB}},
          {binding: 2, resource: {buffer: gpuMatrixC}},
      ]
  });

  // Slide 4a: GPU program source.
  const glslSource = `#version 450
      layout(std430, set = 0, binding = 0) readonly buffer MatrixA {
          float data[];
      } A;
      layout(std430, set = 0, binding = 1) readonly buffer MatrixB {
          float data[];
      } B;
      layout(std430, set = 0, binding = 2) buffer MatrixC {
          float data[];
      } C;
      layout(local_size_x = ${localSize}, local_size_y = ${localSize}) in;

      void main() {
          uvec2 resultCell = gl_GlobalInvocationID.xy;
          uint resultIndex = resultCell.y + resultCell.x * ${matrixDimension};

          float result = 0.0f;
          for (uint i = 0; i < ${matrixDimension}; i++) {
              uint aCell = i + resultCell.x * ${matrixDimension};
              uint bCell = resultCell.y + i * ${matrixDimension};
              result += A.data[aCell] * B.data[bCell];
          }
          C.data[resultIndex] = result;
      }`;

  const computeShaderCode = glslang.compileGLSL(glslSource, "compute");

  // Slide 4b: Compile the GPU program.
  const computePipeline = device.createComputePipeline({
      layout: pipelineLayout,
      computeStage: {
          module: device.createShaderModule({
              code: computeShaderCode
          }),
          entryPoint: "main"
      }
  });

  // Slide 5: Encode the compute commands.
  const commandEncoder = device.createCommandEncoder();

  const passEncoder = commandEncoder.beginComputePass();
  passEncoder.setPipeline(computePipeline);
  passEncoder.setBindGroup(0, bindGroup);
  passEncoder.dispatch(matrixDimension / localSize, matrixDimension / localSize);
  passEncoder.endPass();

  // Slide 6: Encode the readback commands.
  const gpuReadBuffer = device.createBuffer({
      size: matrixSize,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
  });

  commandEncoder.copyBufferToBuffer(
      gpuMatrixC, 0,
      gpuReadBuffer, 0,
      matrixSize
  );

  // Slide 7: Submit work to the GPU.
  await setStatus("Computing on the GPU");
  const timeBefore = window.performance.now();

  const gpuCommands = commandEncoder.finish();
  device.getQueue().submit([gpuCommands]);

  const cpuMatrixC = await gpuReadBuffer.mapReadAsync();

  const elapsedTime = window.performance.now() - timeBefore;
  await setStatus("GPU finished");

  const resultArray = new ArrayBuffer(cpuMatrixC.byteLength);
  const result = new Float32Array(resultArray);
  result.set(new Float32Array(cpuMatrixC));

  return [result, elapsedTime];
}

async function computeOnCPU(matrixA, matrixB) {
  const resultArray = new ArrayBuffer(matrixA.length * 4);
  const result = new Float32Array(resultArray);

  const timeBefore = window.performance.now();
  await setStatus("Computing on the GPU");

  for (let resultX = 0; resultX < matrixDimension; resultX ++) {
      for (let resultY = 0; resultY < matrixDimension; resultY ++) {
          let sum = 0.0;

          for (let i = 0; i < matrixDimension; i++) {
              const aCell = i + resultX * matrixDimension;
              const bCell = resultY + i * matrixDimension;
              sum += matrixA[aCell] * matrixB[bCell];
          }

          const resultCell = resultY + resultX * matrixDimension;
          result[resultCell] = sum;
      }

      if (resultX % 10 === 0) {
          await setStatus("CPU computed row " + resultX);
      }
  }

  const elapsedTime = window.performance.now() - timeBefore;
  await setStatus("CPU finished");

  return [result, elapsedTime];
}

function randomFloats(elementCount) {
  const matrix = [];
  for (let i = 0; i < elementCount; i++) {
      matrix.push(Math.random() * 10);
  }
  return matrix;
}

async function benchmark() {
  console.log('Benchmark started..')
  matrixDimension = document.getElementById("dimension").value;
  matrixElements = matrixDimension * matrixDimension;
  if (matrixDimension > 2048) {alert("don't push it!"); return;}

  document.getElementById("correctness").textContent = "";

  const matrixA = randomFloats(matrixElements);
  const matrixB = randomFloats(matrixElements);

  console.log('Compute On GPU..')
  const [gpuResult, gpuTime] = await computeOnGPU(matrixA, matrixB);
  document.getElementById("gputime").textContent = (gpuTime / 1000).toFixed(3) + "s";

  const [cpuResult, cpuTime] = await computeOnCPU(matrixA, matrixB);
  document.getElementById("cputime").textContent = (cpuTime / 1000).toFixed(3) + "s";

  await setStatus("Computing correctness");

  let correct = true;
  for (let i = 0; i < matrixElements; i++) {
      if (Math.abs(1.0 - (gpuResult[i] / cpuResult[i])) > 0.00001) {
          correct = false;
      }
  }

  if (correct) {
      document.getElementById("correctness").textContent = "Computations match!";
  } else {
      document.getElementById("correctness").textContent = "Computations don't match (float addition issue?)";
  }
  await setStatus("Done");
}

document.getElementById('benchmark').onclick = () => { benchmark() }
