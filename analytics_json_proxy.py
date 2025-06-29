#!/usr/bin/env python3
"""
Analytics JSON Proxy Server
Accepts JSON requests and provides mock responses for testing
"""

from flask import Flask, request, jsonify
from flask_cors import CORS
import logging
from datetime import datetime
import uuid

app = Flask(__name__)
CORS(app)  # Enable CORS for all routes

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@app.route('/api/event', methods=['POST', 'OPTIONS'])
def proxy_event():
    """Handle analytics event from frontend"""
    if request.method == 'OPTIONS':
        # Handle preflight request
        return '', 204
    
    try:
        data = request.get_json()
        logger.info(f"Received analytics event: {data}")
        
        # Extract event type
        event_type = list(data.get('event_type', {}).keys())[0] if data.get('event_type') else 'unknown'
        
        # Generate mock response
        response = {
            "success": True,
            "session_id": str(uuid.uuid4()),
            "event_type": event_type,
            "timestamp": datetime.utcnow().isoformat(),
            "message": f"Event '{event_type}' received successfully (JSON mock mode)"
        }
        
        logger.info(f"Sending response: {response}")
        return jsonify(response), 200
        
    except Exception as e:
        logger.error(f"Error processing request: {str(e)}")
        return jsonify({
            "success": False,
            "error": str(e),
            "message": "Failed to process analytics event"
        }), 500

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({
        "status": "healthy",
        "service": "analytics-json-proxy",
        "timestamp": datetime.utcnow().isoformat()
    })

@app.route('/', methods=['GET'])
def index():
    """Root endpoint"""
    return jsonify({
        "service": "Analytics JSON Proxy",
        "version": "1.0.0",
        "endpoints": {
            "/api/event": "POST - Submit analytics event",
            "/health": "GET - Health check"
        }
    })

if __name__ == '__main__':
    logger.info("Starting Analytics JSON Proxy on port 6691...")
    app.run(host='0.0.0.0', port=6691, debug=True) 