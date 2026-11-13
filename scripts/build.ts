import { copyFileSync, mkdirSync, statSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const root = fileURLToPath(new URL('../', import.meta.url))
const targets = new Map([
  ['x86_64-pc-windows-msvc', 'x64'],
  ['aarch64-pc-windows-msvc', 'arm64'],
])
const args = process.argv.slice(2)
if (args.length && (args.length !== 2 || args[0] !== '--target')) {
  throw new Error('Usage: node scripts/build.ts [--target <Windows MSVC target>]')
}
const target =
  args[1] ??
  process.env.CARGO_BUILD_TARGET ??
  [...targets].find(([, architecture]) => architecture === process.arch)?.[0]
const arch = target === undefined ? undefined : targets.get(target)
if (process.platform !== 'win32' || target === undefined || arch === undefined) {
  throw new Error('Build on Windows for x86_64-pc-windows-msvc or aarch64-pc-windows-msvc')
}

const result = spawnSync('cargo', ['build', '--release', '--locked', '--target', target], {
  cwd: root,
  stdio: 'inherit',
  windowsHide: true,
  env: { ...process.env, CARGO_TARGET_DIR: path.join(root, 'target') },
})
if (result.error) throw result.error
if (result.status !== 0) process.exit(result.status ?? 1)

mkdirSync(path.join(root, 'native'), { recursive: true })
const destination = path.join(root, 'native', `process-start-time.win32-${arch}.node`)
copyFileSync(
  path.join(root, 'target', target, 'release', 'napi_windows_process_start_time.dll'),
  destination,
)
console.log(`${path.relative(root, destination)}: ${statSync(destination).size} bytes`)
