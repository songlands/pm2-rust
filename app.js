const http = require('http');

const PORT = process.env.PORT || 3000;
const HOST = process.env.HOST || '0.0.0.0';

const server = http.createServer((req, res) => {
    const timestamp = new Date().toISOString();
    
    console.log(`[${timestamp}] ${req.method} ${req.url}`);
    
    res.writeHead(200, { 'Content-Type': 'application/json' });
    
    const response = {
        status: 'ok',
        timestamp: timestamp,
        pid: process.pid,
        uptime: process.uptime(),
        memory: process.memoryUsage(),
        request: {
            method: req.method,
            url: req.url,
            headers: req.headers
        }
    };
    
    res.end(JSON.stringify(response, null, 2));
});

server.listen(PORT, HOST, () => {
    console.log(`Server running at http://${HOST}:${PORT}/`);
    console.log(`Process PID: ${process.pid}`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
    console.log('SIGTERM received, shutting down gracefully');
    server.close(() => {
        console.log('Server closed');
        process.exit(0);
    });
});

process.on('SIGINT', () => {
    console.log('SIGINT received, shutting down gracefully');
    server.close(() => {
        console.log('Server closed');
        process.exit(0);
    });
});
