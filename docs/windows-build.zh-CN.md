# 使用 GitHub Actions 构建 Windows 测试安装包

`Windows development build` 工作流仅在 `Sparrow2333/BongoCat` 中运行。
推送 `feat/windows-public-room` 分支会通过仓库统一的打包入口启动构建，
生成 Windows x64 Development 安装包，无需提供发布签名密钥。
Development 的应用数据与正式版隔离。

1. 在你自己的 fork 中打开 **Actions → Windows development build**。
2. 打开所需提交对应的运行记录，等待构建成功。
3. 在 **Artifacts** 中下载 **BongoCat-Windows-x64-development**，
   或点击该次运行摘要中的下载链接。
4. 解压下载文件，在 Windows 10/11 x64 上运行其中的 EXE。

产物保留七天。需要重新构建同一次提交时，可在对应运行记录中选择
**Re-run all jobs**。工作流也声明了手动触发入口；工作流存在于默认分支后，
GitHub 才会在页面中显示 **Run workflow** 按钮。

此工作流不发布 Release，也不创建 PR。构建成功后，仍须在 Windows 电脑上
验证联机、显示缩放和运行稳定性。
