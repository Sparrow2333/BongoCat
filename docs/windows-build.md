# Build the Windows single EXE on GitHub Actions

The `Windows development build` workflow runs only in `Sparrow2333/BongoCat`.
Pushing the `feat/windows-public-room` branch starts a build using the repository's
standard packaging entry point. It creates a Windows x64 Development single EXE
without requiring release signing keys. Development application data is separate
from the production version.

1. Open **Actions → Windows development build** in your fork.
2. Open the run for the required commit and wait for it to succeed.
3. Download **BongoCat-Windows-x64-development** from **Artifacts**, or use the
   download link in the run summary.
4. Extract the download and run its EXE directly on Windows 10/11 x64. No installer
   or sidecar model directory is needed.

Artifacts are retained for seven days. To rebuild the same commit, use **Re-run
all jobs** on its run. The workflow also declares a manual trigger; GitHub shows
the **Run workflow** button when the workflow exists on the default branch.

This workflow does not publish a release or open a pull request. A successful
build does not replace multiplayer, display scaling, or stability testing on a
Windows computer.

To build locally on Windows, run from the repository root:

```text
just build --environment development --target x86_64-pc-windows-msvc --standalone
```

The output is `target/package/BongoCat_<version>_x64_standalone.exe`. Omit
`--environment development` for a production environment build. Preset models are
extracted to a private temporary directory at startup and cleaned up on exit.
Configuration and imported models remain in the application data directory. This
version does not use installer-based automatic updates; quit the application and
download the new EXE to update it. Windows runtime dependencies still need
verification on a clean system.

The workflow copies only the EXE to a new directory to verify its embedded models.
This does not replace real window, input, and GPU acceptance testing.
