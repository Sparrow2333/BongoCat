# 使用 GitHub Actions 构建 Windows 单文件 EXE

`Windows development build` 工作流仅在 `Sparrow2333/BongoCat` 中运行。
推送 `feat/windows-public-room` 分支会通过仓库统一的打包入口启动构建，
生成 Windows x64 Development 单文件 EXE，无需提供发布签名密钥。
Development 的应用数据与正式版隔离。

1. 在你自己的 fork 中打开 **Actions → Windows development build**。
2. 打开所需提交对应的运行记录，等待构建成功。
3. 在 **Artifacts** 中下载 **BongoCat-Windows-x64-development**，
   或点击该次运行摘要中的下载链接。
4. 解压下载文件，在 Windows 10/11 x64 上直接运行其中的 EXE，无需安装或旁置模型文件夹。

产物保留七天。需要重新构建同一次提交时，可在对应运行记录中选择
**Re-run all jobs**。工作流也声明了手动触发入口；工作流存在于默认分支后，
GitHub 才会在页面中显示 **Run workflow** 按钮。

此工作流不发布 Release，也不创建 PR。构建成功后，仍须在 Windows 电脑上
验证联机、显示缩放和运行稳定性。

本地 Windows 构建（工作目录为仓库根目录）：

```text
just build --environment development --target x86_64-pc-windows-msvc --standalone
```

产物为 `target/package/BongoCat_<版本号>_x64_standalone.exe`。去掉
`--environment development` 可构建正式环境版本。模型在启动时释放到当前用户临时目录，
退出后清理；配置与导入模型继续保存在应用数据目录。此版本不使用安装器自动更新，
更新时退出应用并下载新版 EXE。现有 Windows 运行库依赖仍需在干净系统上验证。

工作流会把 EXE 单独复制到新目录验证内嵌模型；这不替代窗口、输入和 GPU 实机验收。
