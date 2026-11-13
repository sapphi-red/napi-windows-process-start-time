# napi-windows-process-start-time

A tiny Windows Node-API addon that reads a running process's creation time.
Uses Rust `#![no_std]`, handwritten Node-API bindings, and Win32 directly.
No runtime npm dependencies, Rust crate dependencies, Rust allocator, FFI
framework, PowerShell, or subprocesses.

```js
import { getProcessStartTime } from 'napi-windows-process-start-time'

const startTime = getProcessStartTime(process.pid)
// 'win32:639...' or null
```

CommonJS is also supported:

```js
const { getProcessStartTime } = require('napi-windows-process-start-time')
```

## API

`getProcessStartTime(pid: number): string | null`

Returns a stable `win32:<ticks>` token. Ticks are 100-nanosecond intervals since
0001-01-01 UTC, matching `(Get-Process -Id $pid).StartTime.ToUniversalTime().Ticks`
in .NET. Keep the value as a string to preserve precision.

Returns `null` when the process has exited, does not exist, or cannot be queried
with the current permissions. Invalid arguments throw `TypeError`: `pid` must
be an integer between 0 and 4294967295. PID 0 is valid input but cannot be queried.

A process can exit immediately after a successful query. Callers must account
for that race when using a timestamp to identify a process.

## Platform support

Windows x64 and ARM64 with standard Node.js 20 or newer. The addon uses Node-API
version 1 and resolves its symbols from the host executable. Embedded Node.js
hosts that do not export Node-API symbols are outside the supported scope.

The loader chooses a prebuilt binary by architecture. Published packages need
no Rust installation. The package has no install scripts or binary downloads.
Other operating systems and architectures are rejected when loading the module.

## Development

Install stable Rust and Visual Studio C++ Build Tools with the Windows SDK.
The default build targets the architecture of the Node.js executable:

```sh
npm ci
npm run build
npm test
npm run check
npm run lint
npm run format
```

The npm dependencies are development tools only. Rust's build script runs on the host
with `std`; the shipped addon itself is entirely `no_std` and links only Win32.
The custom DLL entry point does not initialize the C runtime.

To cross-compile ARM64 from a Windows x64 machine, install the Rust target and
the ARM64 MSVC tools before running:

```sh
rustup target add aarch64-pc-windows-msvc
npm run build -- --target aarch64-pc-windows-msvc
```

## Size and packaging

The initial Windows x64 prototype measured 5,120 bytes for the native binary.
Exact sizes depend on the compiler, target, and implementation. The release
profile uses `opt-level = "z"`, LTO, one codegen unit, stripped symbols, and
`panic = "abort"`.

`npm pack` builds the host binary before packing. To prepare a package containing
both supported architectures, build both targets into `native/`, then run:

```sh
npm pack --ignore-scripts
```

CI builds and tests x64 and ARM64 across Node.js 20, 22, 24, and 26, then produces
an npm tarball containing both binaries. It does not publish the package.

## Implementation

The addon calls `OpenProcess`, `WaitForSingleObject` with a zero timeout,
`GetProcessTimes`, and `CloseHandle`. Handles close on every return path.
Timestamp formatting uses a fixed stack buffer; Node copies the result into
its own JavaScript string. No mutable global state is shared between workers.

## License

MIT
