import { createRequire } from 'node:module'

if (process.platform !== 'win32' || !['x64', 'arm64'].includes(process.arch)) {
  throw new Error(
    `napi-windows-process-start-time supports Windows x64 and ARM64; received ${process.platform}/${process.arch}`,
  )
}

const require = createRequire(import.meta.url)
const filename = `../native/process-start-time.win32-${process.arch}.node`
let binding: { getProcessStartTime(pid: number): bigint | null }
try {
  binding = require(filename)
} catch (cause) {
  throw new Error(
    `Cannot load ${filename}. When using a source checkout, run pnpm run build with Rust and the MSVC build tools installed.`,
    { cause },
  )
}

/**
 * Reads a running Windows process's creation time without spawning a subprocess.
 *
 * Returns `win32:<ticks>`, where ticks are 100-nanosecond intervals since
 * 0001-01-01 UTC, matching .NET DateTime.Ticks. Returns null when the process
 * has exited, does not exist, or cannot be queried with the current permissions.
 *
 * @throws {TypeError} If pid is not an integer between 0 and 4294967295.
 */
export function getProcessStartTime(pid: number): `win32:${string}` | null {
  if (!Number.isInteger(pid) || pid < 0 || pid > 0xffffffff) {
    throw new TypeError('pid must be a uint32')
  }

  const ticks = binding.getProcessStartTime(pid)
  return ticks === null ? null : `win32:${ticks}`
}
