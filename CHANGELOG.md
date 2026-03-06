# 更新日志

## [0.1.0] - 2026-03-06

### 新增功能

#### pm2 show 命令
- 支持 `pm2 show <name>` 通过进程名称查看详情
- 支持 `pm2 show <id>` 通过进程 ID 查看详情（支持短 ID 前缀匹配）

#### pm2 stop 命令
- 支持 `pm2 stop <name>` 通过进程名称停止
- 支持 `pm2 stop <id>` 通过进程 ID 停止
- 支持 `pm2 stop all` 停止所有进程

#### pm2 monit 命令
- 实现实时监控功能
- 支持按 `Ctrl+C` 优雅退出
- 显示进程详情：PID、CPU、内存、运行时间、重启次数
- 美观的界面（带颜色状态图标、时间戳）
- 每秒自动刷新

#### pm2 list 命令
- 添加 PID 列显示
- 使用圆角表格样式
- 列宽自适应内容长度
- 数字列（PID、CPU、内存）右对齐
- 状态列根据状态显示不同颜色

#### 构建脚本
- 新增 `build.sh` 构建脚本
- 自动清理旧构建
- 构建完成后将二进制文件复制到项目根目录

### 代码优化

#### 清理未使用的代码
- **main.rs**: 移除未使用的 `error` import
- **cli/mod.rs**: 移除未使用的 `pub use` 导出
- **config/mod.rs**: 移除未使用的 `ConfigParser` 导出
- **daemon/mod.rs**: 为未使用的 Daemon 相关代码添加 `#[allow(dead_code)]`
- **daemon/server.rs**: 为 DaemonServer 添加 `#[allow(dead_code)]`
- **log/mod.rs**: 移除未使用的 imports
- **log/rotator.rs**: 移除未使用的 imports，优化代码注释
- **process/mod.rs**: 移除未使用的 imports
- **process/manager.rs**: 移除未使用的 imports
- **process/state.rs**: 为未使用的方法添加 `#[allow(dead_code)]`

#### 依赖更新
- 新增 `ctrlc = "3.4"` 依赖用于 monit 命令的信号处理

### Bug 修复

#### pm2 list 显示问题
- 修复 status 列和 restart 列不对齐的问题
- 原因：ANSI 颜色代码影响 tabled 列宽计算
- 解决方案：使用 tabled 库的 Color 功能替代 colored 库的颜色

### 文件变更

```
src/main.rs              - 移除未使用的 import
src/cli/mod.rs           - 移除未使用的导出
src/cli/commands.rs      - 改进 show/stop/monit 命令
src/cli/display.rs       - 改进表格显示，添加颜色支持
src/config/mod.rs        - 移除未使用的导出
src/daemon/mod.rs        - 添加 dead_code 标记
src/daemon/server.rs     - 添加 dead_code 标记
src/log/mod.rs           - 移除未使用的代码
src/log/rotator.rs       - 清理代码
src/process/mod.rs       - 移除未使用的 imports
src/process/manager.rs   - 移除未使用的 imports
src/process/state.rs     - 添加 dead_code 标记
Cargo.toml               - 新增 ctrlc 依赖
build.sh                 - 新增构建脚本
CHANGELOG.md             - 新增更新日志
```

### 构建结果

```
Binary: ./pm2
Size:   4.6M
Status: 无 warnings，无 errors
```

## [0.1.1] - 2026-03-06

### 新增功能

#### 单元测试
- 为 `process` 模块添加单元测试
- 为 `state` 模块添加单元测试
- 为 `manager` 模块添加单元测试
- 为 `log` 模块添加单元测试
- 为 `config` 模块添加单元测试

### 代码优化

#### 测试文件结构
- **config/mod.rs**: 将内联测试移至 `config/tests.rs`
- **config/parser.rs**: 将内联测试移至 `config/tests.rs`
- 统一测试文件结构，便于维护

#### README.md 更新
- 更新项目结构，添加 `src/daemon` 和 `src/log` 目录文件
- 移除 `examples` 目录和测试文件的展示
- 保持项目结构简洁清晰

### Bug 修复

#### Git 过滤问题
- 修复 `pm2` 文件无法被 `.gitignore` 过滤的问题
- 解决方案：使用 `git rm --cached pm2` 移除 git 跟踪

### 文件变更

```
src/config/tests.rs        - 新增配置模块测试
src/config/mod.rs          - 移除内联测试，引用外部测试
src/config/parser.rs       - 移除内联测试
src/process/tests.rs       - 新增进程模块测试
src/process/state_tests.rs - 新增状态模块测试
src/log/tests.rs           - 新增日志模块测试
README.md                  - 更新项目结构
CHANGELOG.md               - 更新修改记录
```

### 构建结果

```
Binary: ./pm2
Size:   4.6M
Status: 无 warnings，无 errors
```
