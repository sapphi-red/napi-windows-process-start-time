import assert from 'node:assert/strict'
import { execFileSync, spawn } from 'node:child_process'
import { once } from 'node:events'
import { createRequire } from 'node:module'
import { Worker } from 'node:worker_threads'
import test from 'node:test'
import { getProcessStartTime } from 'napi-windows-process-start-time'

const require = createRequire(import.meta.url)

test('ESM and CommonJS expose the same function', () => {
  assert.equal(require('napi-windows-process-start-time').getProcessStartTime, getProcessStartTime)
})

test('the current process has a stable, precise timestamp', () => {
  const start = getProcessStartTime(process.pid)
  assert.match(start, /^win32:\d+$/)
  assert.equal(getProcessStartTime(process.pid), start)
})

test('timestamps match PowerShell DateTime ticks', { timeout: 15_000 }, () => {
  const ticks = execFileSync('powershell.exe', [
    '-NoLogo', '-NoProfile', '-NonInteractive', '-Command',
    `(Get-Process -Id ${process.pid} -ErrorAction Stop).StartTime.ToUniversalTime().Ticks`,
  ], { encoding: 'utf8', windowsHide: true, timeout: 10_000 }).trim()
  assert.equal(getProcessStartTime(process.pid), `win32:${ticks}`)
})

test('invalid arguments throw TypeError', () => {
  assert.throws(() => getProcessStartTime(), TypeError)
  for (const input of [undefined, null, '1', {}, [], true, 1n, Symbol(), -1, 1.5, 2 ** 32, NaN, Infinity]) {
    assert.throws(() => getProcessStartTime(input), TypeError)
  }
})

test('unqueryable and nonexistent processes return null', () => {
  assert.equal(getProcessStartTime(0), null)
  assert.equal(getProcessStartTime(0xffffffff), null)
})

test('a child process has a timestamp until it exits', { timeout: 15_000 }, async (t) => {
  const child = spawn(process.execPath, [
    '-e', "setInterval(() => {}, 1000); process.send('ready')",
  ], { stdio: ['ignore', 'ignore', 'inherit', 'ipc'], windowsHide: true })
  const exited = once(child, 'exit')
  t.after(async () => {
    if (child.exitCode === null && child.signalCode === null) child.kill()
    await exited
  })
  await once(child, 'message')
  const start = getProcessStartTime(child.pid)
  assert.match(start, /^win32:\d+$/)
  assert.equal(getProcessStartTime(child.pid), start)
  child.kill()
  await exited
  assert.equal(getProcessStartTime(child.pid), null)
})

test('concurrent workers load and call the addon independently', { timeout: 30_000 }, async (t) => {
  const expected = getProcessStartTime(process.pid)
  const filename = require.resolve('napi-windows-process-start-time')
  await Promise.all(Array.from({ length: 16 }, async () => {
    const worker = new Worker(`
      const { parentPort, workerData } = require('node:worker_threads')
      const { getProcessStartTime } = require(workerData)
      for (let i = 0; i < 1000; i++) getProcessStartTime(process.pid)
      parentPort.postMessage(getProcessStartTime(process.pid))
    `, { eval: true, workerData: filename })
    t.after(() => worker.terminate())
    const exited = once(worker, 'exit')
    const [value] = await once(worker, 'message')
    assert.equal(value, expected)
    assert.deepEqual(await exited, [0])
  }))
})
