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

The package is ESM-only.

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
version 6 and resolves its symbols from the host executable. Embedded Node.js
hosts that do not export Node-API symbols are outside the supported scope.

The loader chooses a prebuilt binary by architecture. Published packages need
no Rust installation. The package has no install scripts or binary downloads.
Other operating systems and architectures are rejected when loading the module.

## Development

Install stable Rust and Visual Studio C++ Build Tools with the Windows SDK.
The default build targets the architecture of the Node.js executable:

```sh
pnpm install --frozen-lockfile
pnpm run build
pnpm test
pnpm run check
pnpm run lint
pnpm run format
```

The pnpm dependencies are development tools only. Rust's build script runs on the host
with `std`; the shipped addon itself is entirely `no_std` and links only Win32.
The addon links with `/NOENTRY` because it needs no DLL entry point or C runtime
initialization. Node.js initializes the addon through `napi_register_module_v1`.

Clippy follows [Rolldown's lint rules](https://github.com/rolldown/rolldown/blob/main/Cargo.toml),
including its pedantic, nursery, and restriction lint settings. Run it on the library
with `cargo clippy --lib --locked -- -D warnings`; Rust's test harness uses `std`
and conflicts with this addon's custom panic handler.

To cross-compile ARM64 from a Windows x64 machine, install the Rust target and
the ARM64 MSVC tools before running:

```sh
rustup target add aarch64-pc-windows-msvc
pnpm run build --target aarch64-pc-windows-msvc
```

## Size and packaging

The Windows x64 native binary measured 3,584 bytes after moving PID validation
and timestamp formatting into JavaScript (initial prototype: 5,120 bytes).
Exact sizes depend on the compiler, target, and implementation. The release
profile uses `opt-level = "z"`, LTO, one codegen unit, stripped symbols, and
`panic = "abort"`.

`pnpm pack` builds the host binary before packing. To prepare a package containing
both supported architectures, build both targets into `native/`, then run:

```sh
pnpm pack --ignore-scripts
```

CI builds and tests x64 and ARM64 across Node.js 20, 22, 24, and 26, then produces
an npm tarball containing both binaries. After packaging succeeds, CI uploads that
tarball to [pkg.pr.new](https://github.com/stackblitz-labs/pkg.pr.new) for preview
installs on pushes, pull requests, and manual workflow runs. Preview install links
appear in the workflow logs and pull request comments.

Install the [pkg.pr.new GitHub App](https://github.com/apps/pkg-pr-new) on the
repository before running the workflow. Previews use the packaged tarball as-is
and do not publish to npm.

## Implementation

The addon calls `OpenProcess`, `WaitForSingleObject` with a zero timeout,
`GetProcessTimes`, and `CloseHandle`. Handles close on every return path.
The JavaScript wrapper validates the PID before calling the native binding.
The binding returns ticks as a BigInt to preserve all 64 bits; the wrapper formats
the public `win32:<ticks>` string. No mutable global state is shared between workers.

## License

MIT
