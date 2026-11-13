import { execFileSync, spawn } from 'node:child_process'
import { once } from 'node:events'
import { Worker } from 'node:worker_threads'
import { expect, test } from 'vitest'
import { getProcessStartTime } from 'napi-windows-process-start-time'

test('the current process has a stable, precise timestamp', () => {
  const start = getProcessStartTime(process.pid)
  expect(start).toMatch(/^win32:\d+$/)
  expect(getProcessStartTime(process.pid)).toBe(start)
})

test('timestamps match PowerShell DateTime ticks', { timeout: 15_000 }, () => {
  const ticks = execFileSync(
    'powershell.exe',
    [
      '-NoLogo',
      '-NoProfile',
      '-NonInteractive',
      '-Command',
      `(Get-Process -Id ${process.pid} -ErrorAction Stop).StartTime.ToUniversalTime().Ticks`,
    ],
    { encoding: 'utf8', windowsHide: true, timeout: 10_000 },
  ).trim()
  expect(getProcessStartTime(process.pid)).toBe(`win32:${ticks}`)
})

test('invalid arguments throw TypeError', () => {
  // @ts-expect-error Exercise runtime validation of a missing PID.
  expect(() => getProcessStartTime()).toThrow(TypeError)
  for (const input of [
    undefined,
    null,
    '1',
    {},
    [],
    true,
    1n,
    Symbol(),
    new Number(1),
    -1,
    -0.5,
    Number.MIN_VALUE,
    1.5,
    2 ** 32 - 0.5,
    2 ** 32,
    Number.MAX_SAFE_INTEGER,
    NaN,
    Infinity,
    -Infinity,
  ]) {
    // @ts-expect-error Exercise runtime validation of invalid PID types and values.
    expect(() => getProcessStartTime(input)).toThrow(TypeError)
  }
})

test('unqueryable and nonexistent processes return null', () => {
  expect(getProcessStartTime(0)).toBeNull()
  expect(getProcessStartTime(-0)).toBeNull()
  expect(getProcessStartTime(0xffffffff)).toBeNull()
})

test(
  'a child process has a timestamp until it exits',
  { timeout: 15_000 },
  async ({ onTestFinished }) => {
    const child = spawn(
      process.execPath,
      ['-e', "setInterval(() => {}, 1000); process.send('ready')"],
      { stdio: ['ignore', 'ignore', 'inherit', 'ipc'], windowsHide: true },
    )
    const exited = once(child, 'exit')
    onTestFinished(async () => {
      if (child.exitCode === null && child.signalCode === null) child.kill()
      await exited
    })
    await once(child, 'message')
    const pid = child.pid
    if (pid === undefined) throw new Error('Child process did not start')
    const start = getProcessStartTime(pid)
    expect(start).toMatch(/^win32:\d+$/)
    expect(getProcessStartTime(pid)).toBe(start)
    child.kill()
    await exited
    expect(getProcessStartTime(pid)).toBeNull()
  },
)

test(
  'concurrent workers load and call the addon independently',
  { timeout: 30_000 },
  async ({ onTestFinished }) => {
    const expected = getProcessStartTime(process.pid)
    await Promise.all(
      Array.from({ length: 16 }, async () => {
        const worker = new Worker(new URL('./fixtures/worker.mjs', import.meta.url))
        onTestFinished(async () => {
          await worker.terminate()
        })
        const exited = once(worker, 'exit')
        const [value] = await once(worker, 'message')
        expect(value).toBe(expected)
        expect(await exited).toEqual([0])
      }),
    )
  },
)
