# PM2 示例程序

本目录包含三个不同语言的HTTP服务器示例，用于测试Rust PM2工具。

## 示例程序

### 1. Go HTTP服务器
- **文件**: `go-server/main.go`
- **默认端口**: 3001
- **功能**: 提供HTTP API，返回进程信息、内存使用情况等

### 2. Node.js HTTP服务器
- **文件**: `node-server/app.js`
- **默认端口**: 3002
- **功能**: 提供HTTP API，返回进程信息、内存使用情况等

### 3. Python HTTP服务器
- **文件**: `python-server/app.py`
- **默认端口**: 3003
- **功能**: 提供HTTP API，返回进程信息、内存使用情况等

## 使用方法

### 编译Go程序
```bash
cd go-server
go build -o go-server main.go
```

### 启动Node.js程序
```bash
cd node-server
node app.js
```

### 启动Python程序
```bash
cd python-server
python3 app.py
```

## 使用PM2管理

### 启动Go服务器
```bash
./target/release/pm2 start examples/go-server/go-server --name go-server
```

### 启动Node.js服务器
```bash
./target/release/pm2 start examples/node-server/app.js --name node-server
```

### 启动Python服务器
```bash
./target/release/pm2 start examples/python-server/app.py --name python-server
```

### 查看所有进程
```bash
./target/release/pm2 list
```

### 停止进程
```bash
./target/release/pm2 stop <name>
```

### 重启进程
```bash
./target/release/pm2 restart <name>
```

### 删除进程
```bash
./target/release/pm2 delete <name>
```

### 查看进程详情
```bash
./target/release/pm2 show <name>
```

## 测试API

所有服务器都提供相同的API接口：

### GET /
返回JSON格式的进程信息，包括：
- status: 服务状态
- timestamp: 当前时间戳
- pid: 进程ID
- uptime: 运行时间
- memory: 内存使用情况
- request: 请求信息（方法、URL、headers）

### 示例请求
```bash
curl http://localhost:3001/
curl http://localhost:3002/
curl http://localhost:3003/
```

## 环境变量

所有服务器都支持以下环境变量：

- `PORT`: 监听端口（默认值：Go=3001, Node.js=3002, Python=3003）
- `HOST`: 监听地址（默认值：0.0.0.0）

### 示例
```bash
PORT=8080 HOST=127.0.0.1 ./target/release/pm2 start examples/node-server/app.js --name node-server
```

## 信号处理

所有服务器都正确处理以下信号：
- `SIGTERM`: 优雅关闭
- `SIGINT`: 中断信号（Ctrl+C）
