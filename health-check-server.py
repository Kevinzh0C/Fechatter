#!/usr/bin/env python3

"""
Simple HTTP server for global health check
Serves comprehensive health status on port 9999
CORS headers managed by nginx gateway
"""

import http.server
import socketserver
import subprocess
import json
import sys
from urllib.parse import urlparse

class HealthCheckHandler(http.server.BaseHTTPRequestHandler):
    
    def do_GET(self):
        """Handle GET requests for health check"""
        if self.path in ['/', '/health']:
            try:
                # Execute the global health check script
                result = subprocess.run(
                    ['/usr/local/bin/global-health-check.sh'],
                    capture_output=True,
                    text=True,
                    timeout=10
                )
                
                if result.returncode == 0:
                    # Clean up the output (remove any extra text)
                    output = result.stdout.strip()
                    # Remove the "EOF" warning if present
                    if 'EOF' in output:
                        lines = output.split('\n')
                        json_lines = []
                        for line in lines:
                            if not line.startswith('/usr/local/bin/global-health-check.sh'):
                                json_lines.append(line)
                        output = '\n'.join(json_lines).strip()
                    
                    # Validate JSON
                    try:
                        json.loads(output)
                        self.send_response(200)
                        self.send_header('Content-Type', 'application/json')
                        self.end_headers()
                        self.wfile.write(output.encode('utf-8'))
                    except json.JSONDecodeError:
                        self.send_error_response("Invalid JSON output from health check")
                else:
                    self.send_error_response(f"Health check script failed: {result.stderr}")
                    
            except subprocess.TimeoutExpired:
                self.send_error_response("Health check timed out")
            except Exception as e:
                self.send_error_response(f"Health check error: {str(e)}")
        
        elif self.path == '/ping':
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            self.wfile.write(b'pong')
        
        else:
            self.send_response(404)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"error": "Not found"}')
    
    def do_OPTIONS(self):
        """Handle preflight CORS requests - managed by nginx"""
        self.send_response(200)
        self.end_headers()
    
    def send_error_response(self, message):
        """Send error response in JSON format"""
        self.send_response(500)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        error_response = {
            "status": "error",
            "message": message,
            "timestamp": int(__import__('time').time() * 1000)
        }
        self.wfile.write(json.dumps(error_response).encode('utf-8'))
    
    def log_message(self, format, *args):
        """Suppress default logging"""
        pass

def main():
    PORT = 9999
    
    try:
        with socketserver.TCPServer(("", PORT), HealthCheckHandler) as httpd:
            print(f"Health check server running on port {PORT}")
            print(f"Endpoints:")
            print(f"  GET http://localhost:{PORT}/health - Comprehensive health check")
            print(f"  GET http://localhost:{PORT}/ping - Simple ping")
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down health check server...")
    except Exception as e:
        print(f"Error starting server: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 