# napi-windows-process-start-time

A tiny Windows Node-API addon that reads a running process's creation time.

```js
import { getProcessStartTime } from 'napi-windows-process-start-time'

const startTime = getProcessStartTime(process.pid)
// 'win32:639...' or null
```

## API

`getProcessStartTime(pid: number): string | null`

Returns a stable `win32:<ticks>` token. Ticks are 100-nanosecond intervals since 0001-01-01 UTC, matching `(Get-Process -Id $pid).StartTime.ToUniversalTime().Ticks` in .NET. The value is kept as a string to preserve precision.

Returns `null` when the process has exited, does not exist, or cannot be queried with the current permissions. Invalid arguments throw `TypeError`: `pid` must be an integer between 0 and 4294967295. PID 0 is valid input but cannot be queried.

A process can exit immediately after a successful query. Callers must account for that race when using a timestamp to identify a process.

## Platform support

Windows x64 and ARM64 with Node.js 20+.

## Development

```sh
pnpm install --frozen-lockfile
pnpm run build
pnpm test
pnpm run check
pnpm run lint
pnpm run format
```

## License

MIT
