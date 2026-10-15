# ADR-0091: Windows Single Executable

状态：已接受（2026-10-15）；Windows 构建与实机验收待完成

## 决策

- 统一打包入口增加 `--standalone`，仅接受 Windows x64，不与 `--formats` 混用。原有安装器与发布更新载荷保持原协议。
- 单文件构建启用 `bongocat-app/standalone`，构建期将三个预置模型的文件直接嵌入应用，不新增 launcher、service 或第二套 runtime。不嵌入工作目录杂项，拒绝符号链接。
- 启动时将编译期资源写入当前用户的随机私有临时目录。资源 owner 存活到 runtime、音频与 renderer 全部退出之后，再自动清理；启动失败同样释放。强杀可能遗留临时目录，不能宣称该路径已完成长期 soak。
- 数据继续使用构建环境隔离的既有目录，不写 EXE 所在目录。单文件不代表配置随 EXE 携带；不修改权限提示。
- 单文件模式禁用安装器自动更新，不允许静默转换成安装版。更新需要重新下载 EXE。
- 复用现有 Cubism Core MD 静态库与 Windows 系统依赖；单文件分发不等于已证明无 VC 运行库依赖。干净 Windows 上验证 DLL 依赖是交付门禁，未通过前不得宣称完全自包含。
- Development Windows 工作流改为产出单文件 EXE，并将其复制到没有旁置资源的新目录执行内嵌资源 smoke；该检查不启动 UI、输入或 GPU。签名必须在最终 EXE 完成后执行；本路径不生成 NSIS 更新 manifest。

## 验证与退出条件

自动化覆盖三个预置模型包校验、资源路径边界、失败与正常析构、参数互斥与平台限制、Development/Production 更新禁用。退出条件还包括 Windows x64 构建、干净 Windows 10/11 的 DLL 依赖及离仓启动、模型切换、配置保留、登录启动项、正常退出和临时资源清理验证。
