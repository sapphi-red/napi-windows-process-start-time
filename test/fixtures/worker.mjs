import { parentPort } from 'node:worker_threads'
import { getProcessStartTime } from 'napi-windows-process-start-time'

for (let i = 0; i < 1000; i++) getProcessStartTime(process.pid)
parentPort.postMessage(getProcessStartTime(process.pid))
