this.eventSource.onopen = () => {
  this.isConnected = true;
  this.reconnectAttempts = 0;
  // Emit status update to subscribers
  this.emit('status', this.getPerformanceStats());
  if (import.meta.env.DEV) {
    console.log('⚡ [SSE] Connected successfully');
  }
};

this.eventSource.onerror = (error) => {
  if (import.meta.env.DEV) {
    console.warn('⚠️ [SSE] Connection error:', error);
  this.isConnected = false;
  // Emit status update on error
  this.emit('status', this.getPerformanceStats());
  this.scheduleReconnect();
}; 