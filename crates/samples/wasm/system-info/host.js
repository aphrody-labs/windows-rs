// A minimal JavaScript host for this sample. It supplies the Win32 and NT imports the module
// references, following the contract in docs/wasm.md, and checks what the exports return.
//
//   cargo build --release --target wasm32-unknown-unknown -p wasm-system-info
//   bun crates/samples/wasm/system-info/host.js target/wasm32-unknown-unknown/release/wasm_system_info.wasm

const path = Bun.argv[2];
if (!path) {
  throw new Error("usage: bun host.js <wasm_system_info.wasm>");
}

const module = await WebAssembly.compile(await Bun.file(path).arrayBuffer());

// Each Win32/NT function is imported from a module named after its DLL.
const imports = WebAssembly.Module.imports(module)
  .map(({ module, name, kind }) => `${kind} ${module} ${name}`)
  .sort();
const expected = ["function kernel32.dll GetTickCount64", "function ntdll.dll NtQuerySystemInformation"];
if (JSON.stringify(imports) !== JSON.stringify(expected)) {
  throw new Error(`unexpected imports: ${imports.join(", ")}`);
}

// Above 32 bits on purpose: a 64-bit result crosses the boundary as a BigInt.
const TICKS = 0x1_0000_0001n;
const PROCESSORS = 8;

const STATUS_SUCCESS = 0;
const STATUS_INVALID_INFO_CLASS = 0xc0000003 | 0;
const STATUS_INFO_LENGTH_MISMATCH = 0xc0000004 | 0;
const SystemBasicInformation = 0;

// SYSTEM_BASIC_INFORMATION has the 32-bit Windows layout on wasm32.
const BASIC_INFORMATION_SIZE = 44;
const NUMBER_OF_PROCESSORS_OFFSET = 40;

let memory;

const instance = await WebAssembly.instantiate(module, {
  "kernel32.dll": {
    GetTickCount64: () => TICKS,
  },
  "ntdll.dll": {
    // Pointers arrive as signed i32 offsets into the module's linear memory.
    NtQuerySystemInformation(informationClass, information, length, returnLength) {
      information >>>= 0;
      returnLength >>>= 0;
      if (informationClass !== SystemBasicInformation) {
        return STATUS_INVALID_INFO_CLASS;
      }
      const view = new DataView(memory.buffer);
      if (returnLength !== 0) {
        view.setUint32(returnLength, BASIC_INFORMATION_SIZE, true);
      }
      if (length >>> 0 < BASIC_INFORMATION_SIZE) {
        return STATUS_INFO_LENGTH_MISMATCH;
      }
      new Uint8Array(memory.buffer, information, BASIC_INFORMATION_SIZE).fill(0);
      view.setInt8(information + NUMBER_OF_PROCESSORS_OFFSET, PROCESSORS);
      return STATUS_SUCCESS;
    },
  },
});

memory = instance.exports.memory;

const uptime = instance.exports.uptime_ms();
const processors = instance.exports.processor_count();

console.log(`imports: ${imports.join(", ")}`);
console.log(`uptime_ms() = ${uptime}, processor_count() = ${processors}`);

if (uptime !== TICKS || processors !== PROCESSORS) {
  throw new Error("unexpected results");
}
