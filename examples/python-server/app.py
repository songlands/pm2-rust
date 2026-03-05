import os
import json
import time
import signal
import sys
from datetime import datetime
from http.server import HTTPServer, BaseHTTPRequestHandler

PORT = int(os.getenv('PORT', 3004))
HOST = os.getenv('HOST', '0.0.0.0')

START_TIME = time.time()

server = None

class RequestHandler(BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.0'
    
    def do_GET(self):
        timestamp = datetime.now().isoformat()
        
        print(f"[{timestamp}] {self.command} {self.path}")
        
        try:
            response = {
                'status': 'ok',
                'timestamp': timestamp,
                'pid': os.getpid(),
                'uptime': time.time() - START_TIME,
                'memory': {
                    'rss': os.getresourcusage(os.RUSAGE_SELF).ru_maxrss,
                },
                'request': {
                    'method': self.command,
                    'url': self.path,
                    'headers': dict(self.headers),
                },
                'python': {
                    'version': sys.version,
                    'platform': sys.platform,
                }
            }
            
            response_str = json.dumps(response, indent=2)
            response_bytes = response_str.encode('utf-8')
            
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(response_bytes)))
            self.end_headers()
            
            self.wfile.write(response_bytes)
            self.wfile.flush()
        except Exception as e:
            print(f"Error handling request: {e}")
            import traceback
            traceback.print_exc()
    
    def do_POST(self):
        self.do_GET()
    
    def do_PUT(self):
        self.do_GET()
    
    def do_DELETE(self):
        self.do_GET()
    
    def log_message(self, format, *args):
        pass

def signal_handler(signum, frame):
    global server
    print(f"Received signal {signum}, shutting down gracefully...")
    if server:
        server.shutdown()
    sys.exit(0)

if __name__ == '__main__':
    server = HTTPServer((HOST, PORT), RequestHandler)
    
    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)
    
    print(f"Server running at http://{HOST}:{PORT}/")
    print(f"Process PID: {os.getpid()}")
    print(f"Python version: {sys.version}")
    print(f"Platform: {sys.platform}")
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        if server:
            server.shutdown()
