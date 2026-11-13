import { createRequire } from 'node:module'

if (process.platform !== 'win32' || !['x64', 'arm64'].includes(process.arch)) {
  throw new Error(
    `napi-windows-process-start-time supports Windows x64 and ARM64; received ${process.platform}/${process.arch}`,
  )
}

const require = createRequire(import.meta.url)
const filename = `./native/process-start-time.win32-${process.arch}.node`
let binding
try {
  binding = require(filename)
} catch (cause) {
  throw new Error(
    `Cannot load ${filename}. When using a source checkout, run pnpm run build with Rust and the MSVC build tools installed.`,
    { cause },
  )
}

export const getProcessStartTime = binding.getProcessStartTime
