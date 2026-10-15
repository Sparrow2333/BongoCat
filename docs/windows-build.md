# Build the Windows test installer on GitHub Actions

The `Windows development build` workflow runs only in `Sparrow2333/BongoCat`.
Pushing the `feat/windows-public-room` branch starts a build using the repository's
standard packaging entry point. It creates a Windows x64 Development installer
without requiring release signing keys. Development application data is separate
from the production version.

1. Open **Actions → Windows development build** in your fork.
2. Open the run for the required commit and wait for it to succeed.
3. Download **BongoCat-Windows-x64-development** from **Artifacts**, or use the
   download link in the run summary.
4. Extract the download and run the EXE on Windows 10/11 x64.

Artifacts are retained for seven days. To rebuild the same commit, use **Re-run
all jobs** on its run. The workflow also declares a manual trigger; GitHub shows
the **Run workflow** button when the workflow exists on the default branch.

This workflow does not publish a release or open a pull request. A successful
build does not replace multiplayer, display scaling, or stability testing on a
Windows computer.
