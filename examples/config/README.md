# PM2 配置文件示例

本目录包含各种格式的 PM2 配置文件示例。

## 配置文件格式

支持以下格式：
- **TOML** (`.toml`) - 推荐使用，语法简洁
- **YAML** (`.yaml`, `.yml`) - 广泛使用，可读性好
- **JSON** (`.json`) - 标准格式，易于解析

## 文件说明

### 单个应用配置

#### TOML 格式
- **文件**: `app.toml`
- **说明**: 单个 Node.js HTTP 服务器配置
- **启动**: `pm2 start app.toml`

#### YAML 格式
- **文件**: `app.yaml`
- **说明**: 单个 Node.js HTTP 服务器配置
- **启动**: `pm2 start app.yaml`

### 生态系统配置（多应用）

#### TOML 格式
- **文件**: `ecosystem.toml`
- **说明**: 包含 Go、Node.js、Python 三个服务器的配置
- **启动**: `pm2 start ecosystem.toml`

#### YAML 格式
- **文件**: `ecosystem.yaml`
- **说明**: 包含 Go、Node.js、Python 三个服务器的配置
- **启动**: `pm2 start ecosystem.yaml`

### 集群模式配置

#### TOML 格式
- **文件**: `cluster.toml`
- **说明**: 两个集群模式应用（api-cluster: 4实例, web-cluster: 2实例）
- **启动**: `pm2 start cluster.toml`

#### YAML 格式
- **文件**: `cluster.yaml`
- **说明**: 两个集群模式应用（api-cluster: 4实例, web-cluster: 2实例）
- **启动**: `pm2 start cluster.yaml`

## 配置选项

### 基本选项
```yaml
name: app-name              # 应用名称
script: app.js              # 脚本路径
instances: 1                # 实例数量
cwd: .                      # 工作目录
```

### 执行模式
```yaml
exec_mode: fork             # fork 模式（默认）
exec_mode: cluster          # cluster 模式
```

### 监控选项
```yaml
watch: false                # 是否监听文件变化
max_memory_restart: 500M    # 最大内存限制
autorestart: true           # 是否自动重启
max_restarts: 15            # 最大重启次数
```

### 日志选项
```yaml
log_file: ./logs/app.log           # 标准输出日志
error_file: ./logs/error.log       # 错误日志
out_file: ./logs/out.log           # 输出日志
merge_logs: false                  # 是否合并日志
```

### 环境变量
```yaml
env:
  NODE_ENV: production
  PORT: "3000"
```

## 使用示例

### 启动单个应用
```bash
# TOML 格式
pm2 start app.toml

# YAML 格式
pm2 start app.yaml
```

### 启动生态系统
```bash
# TOML 格式
pm2 start ecosystem.toml

# YAML 格式
pm2 start ecosystem.yaml
```

### 启动集群模式
```bash
# TOML 格式
pm2 start cluster.toml

# YAML 格式
pm2 start cluster.yaml
```

### 查看进程
```bash
pm2 list
pm2 show app-name
```

### 停止和删除
```bash
pm2 stop app-name
pm2 delete app-name
pm2 stop all
pm2 delete all
```

## 格式对比

### TOML 优点
- 语法简洁，易于编写
- 注释清晰
- 适合配置文件

### YAML 优点
- 广泛使用，可读性好
- 支持复杂数据结构
- 缩进表示层级关系

### JSON 优点
- 标准格式，易于解析
- 支持所有编程语言
- 严格的语法规范

## 注意事项

1. YAML 格式使用缩进表示层级关系，请使用空格而非制表符
2. TOML 格式使用 `[[apps]]` 表示数组元素
3. 环境变量值建议使用字符串格式（加引号）
4. 内存限制格式：`100M`, `1G`, `512K`
5. 端口等数字建议作为字符串处理
