# Unsafe Code Coverage

## Policy

- `#![deny(unsafe_code)]` is set in `src/lib.rs`.
- Unsafe is allowed **only** in `src/platform/windows/*.rs` via per-file
  `#![allow(unsafe_code)]`.
- Every unsafe block wraps Win32 FFI calls; no pointer arithmetic or
  transmutes are used outside of kernel32/advapi32 API contracts.

## Inventory (12 blocks)

| ID  | File           | Line | Win32 API                                            | Risk           | Unit Test | Integration Test |
| --- | -------------- | ---- | ---------------------------------------------------- | -------------- | --------- | ---------------- |
| A1  | attributes.rs  | 25   | CreateFileW, GetFileInformationByHandle, CloseHandle | handle leak    | ✅        | ✅               |
| A2  | attributes.rs  | 70   | GetFileAttributesW                                   | low            | ✅        | ✅               |
| C1  | console.rs     | 6    | GetStdHandle, GetConsoleMode, SetConsoleMode         | low            | ✅        | ✅               |
| C2  | console.rs     | 18   | GetStdHandle, GetConsoleMode                         | low            | ✅        | ✅               |
| C3  | console.rs     | 30   | GetStdHandle, GetConsoleScreenBufferInfo             | zeroed struct  | ✅        | —¹               |
| L1  | locale.rs      | 9    | GetUserDefaultUILanguage                             | minimal        | ✅        | ✅               |
| P1  | permissions.rs | 40   | GetFileSecurityW (2-call pattern)                    | buffer overrun | ✅²       | ✅²              |
| P2  | permissions.rs | 76   | LookupAccountSidW (2-call pattern)                   | buffer overrun | ✅²       | ✅²              |
| P3  | permissions.rs | 129  | GetSecurityDescriptorOwner                           | null deref     | ✅        | ✅               |
| P4  | permissions.rs | 150  | GetSecurityDescriptorGroup                           | null deref     | ✅        | ✅               |
| R1  | reparse.rs     | 33   | CreateFileW, DeviceIoControl, CloseHandle + parse    | OOB, handle    | ✅        | ✅               |
| S1  | streams.rs     | 37   | FindFirstStreamW, FindNextStreamW, FindClose         | handle leak    | ✅        | ✅               |

¹ `get_console_width` not exposed via public `platform::` API; covered by unit test only.
² P1/P2 are internal helpers called transitively by `get_file_owner` / `get_file_group`.

## Why not Miri?

Miri does not support foreign function calls (FFI). All 12 unsafe blocks
invoke Win32 APIs via `windows-sys`, which Miri cannot execute.

Handle-leak detection is approximated by **stress tests** that call each
function 1000–2000 times in a loop. If `CloseHandle` / `FindClose` were
missing, the process would hit the ~16 384 handle limit and the test would
fail with `ERROR_TOO_MANY_OPEN_FILES`.

## Adding new unsafe

1. Add `#![allow(unsafe_code)]` only in the new file under `platform/windows/`.
2. Add the block to this table with a unique ID.
3. Add unit test in the same file + integration test in `tests/windows_unsafe.rs`.
4. CI will verify on `windows-latest` automatically.
