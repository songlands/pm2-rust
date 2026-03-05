const http = require('http');

const PORT = process.env.PORT || 3002;
const HOST = process.env.HOST || '0.0.0.0';

const startTime = Date.now();

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
    console.log(JSON.stringify(response, null, 2))

    res.end(JSON.stringify(response, null, 2));
});

server.listen(PORT, HOST, () => {
    console.log(`Server running at http://${HOST}:${PORT}/`);
    console.log(`Process PID: ${process.pid}`);
    console.log(`Node version: ${process.version}`);
    console.log(`Platform: ${process.platform}`);
    console.log(`Architecture: ${process.arch}`);
});

process.on('SIGTERM', () => {
    console.log('Received SIGTERM, shutting down gracefully...');
    server.close(() => {
        console.log('Server closed');
        process.exit(0);
    });
});

process.on('SIGINT', () => {
    console.log('Received SIGINT, shutting down gracefully...');
    server.close(() => {
        console.log('Server closed');
        process.exit(0);
    });
});
