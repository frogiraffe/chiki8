# External test input policy

`vendor/timendus-chip8-test-suite.tsv` records the exact source, revision, license, checksum, execution schedule, and purpose of each selected conformance input. The runner fetches those ROMs into a disposable directory and verifies them before use; no fetched `.ch8` file is tracked.

The Timendus inputs are classified `test-only-transient`. Release and archive tooling must not include the temporary checkout or fetched ROM bytes. Public documentation media requires a separately approved redistributable source in Phase 07; passing these tests does not authorize treating their captures as public release assets.

The adjacent license copy is the upstream `LICENSE` file at commit `cb24d5595384a80b49ddedae13bec4042b16d41d`.

`vendor/ORACLE-DERIVATION.md` records the independent upstream sources and
deterministic procedure used to approve framebuffer hashes.
