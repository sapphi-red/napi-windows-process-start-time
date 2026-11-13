/**
 * Reads a running Windows process's creation time without spawning a subprocess.
 *
 * Returns `win32:<ticks>`, where ticks are 100-nanosecond intervals since
 * 0001-01-01 UTC, matching .NET DateTime.Ticks. Returns null when the process
 * has exited, does not exist, or cannot be queried with the current permissions.
 *
 * @throws {TypeError} If pid is not an integer between 0 and 4294967295.
 */
export declare function getProcessStartTime(pid: number): `win32:${string}` | null
