/**
 * Message Flow Debugger
 * Tracks complete message flow from UI to backend and back
 */

class MessageFlowDebugger {
  constructor() {
    this.flows = new Map();
    this.currentFlowId = null;
    this.enabled = true;
  }

  startFlow(flowType, context) {
    if (!this.enabled) return;

    const flowId = `${flowType}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    this.currentFlowId = flowId;

    const flow = {
      id: flowId,
      type: flowType,
      context,
      startTime: Date.now(),
      steps: [],
      endTime: null
    };

    this.flows.set(flowId, flow);
    if (import.meta.env.DEV) {
      console.log(`🚀 [FLOW_DEBUG] Started flow: ${flowId}`, context);
    }

    return flowId;
  }

  addStep(stepName, data, flowId = null) {
    if (!this.enabled) return;

    const targetFlowId = flowId || this.currentFlowId;
    if (!targetFlowId) return;

    const flow = this.flows.get(targetFlowId);
    if (!flow) return;

    const step = {
      name: stepName,
      timestamp: Date.now(),
      elapsed: Date.now() - flow.startTime,
      data: this.sanitizeData(data),
      stack: this.getCallStack()
    };

    flow.steps.push(step);
    if (import.meta.env.DEV) {
      console.log(`📍 [FLOW_DEBUG][${targetFlowId}] Step: ${stepName}`, {
        elapsed: `${step.elapsed}ms`,
        data: step.data
      });
    }
  }

  endFlow(flowId = null) {
    if (!this.enabled) return;

    const targetFlowId = flowId || this.currentFlowId;
    if (!targetFlowId) return;

    const flow = this.flows.get(targetFlowId);
    if (!flow) return;

    flow.endTime = Date.now();
    const totalTime = flow.endTime - flow.startTime;

    if (import.meta.env.DEV) {
      console.log(`✅ [FLOW_DEBUG] Completed flow: ${targetFlowId} in ${totalTime}ms`);
      this.printFlowSummary(flow);
    }
  }

  printFlowSummary(flow) {
    console.group(`📊 Flow Summary: ${flow.id}`);
    if (import.meta.env.DEV) {
      console.log(`Type: ${flow.type}`);
      console.log(`Total Time: ${flow.endTime - flow.startTime}ms`);
      console.log(`Steps: ${flow.steps.length}`);
    }

    console.group('Step Details:');
    flow.steps.forEach((step, index) => {
      if (import.meta.env.DEV) {
        console.log(`${index + 1}. ${step.name} (+${step.elapsed}ms)`);
        if (step.data && Object.keys(step.data).length > 0) {
          console.log('   Data:', step.data);
        }
      }
    });
    console.groupEnd();
    console.groupEnd();
  }

  sanitizeData(data) {
    if (!data) return null;

    // Avoid circular references and large objects
    try {
      const sanitized = {};
      for (const key in data) {
        const value = data[key];
        if (typeof value === 'function') {
          sanitized[key] = '[Function]';
        } else if (value instanceof Error) {
          sanitized[key] = value.message;
        } else if (Array.isArray(value)) {
          sanitized[key] = `Array(${value.length})`;
        } else if (typeof value === 'object' && value !== null) {
          sanitized[key] = '[Object]';
        } else {
          sanitized[key] = value;
        }
      }
      return sanitized;
    } catch (error) {
      return { error: 'Failed to sanitize data' };
    }
  }

  getCallStack() {
    try {
      const stack = new Error().stack;
      const lines = stack.split('\n').slice(3, 6); // Get lines 3-6 of call stack
      return lines.map(line => line.trim()).filter(line => line);
    } catch (error) {
      return [];
    }
  }

  findDuplicateMessages(messages) {
    const seen = new Map();
    const duplicates = [];

    messages.forEach((msg, index) => {
      const keys = [
        msg.id && `id:${msg.id}`,
        msg.temp_id && `temp_id:${msg.temp_id}`,
        msg.idempotency_key && `idempotency_key:${msg.idempotency_key}`,
        `content_sender:${msg.content}_${msg.sender_id}`
      ].filter(Boolean);

      keys.forEach(key => {
        if (seen.has(key)) {
          duplicates.push({
            index,
            key,
            message: msg,
            duplicateOf: seen.get(key)
          });
        } else {
          seen.set(key, { index, message: msg });
        }
      });
    });

    if (duplicates.length > 0) {
      if (import.meta.env.DEV) {
        console.warn(`🔴 [FLOW_DEBUG] Found ${duplicates.length} duplicate messages:`, duplicates);
      }
    }

    return duplicates;
  }

  traceMessagePath(messageId) {
    console.group(`🔍 Tracing message path for ID: ${messageId}`);

    // Find all flows containing this message
    const relevantFlows = [];
    this.flows.forEach((flow, flowId) => {
      flow.steps.forEach(step => {
        if (step.data && (
          step.data.messageId === messageId ||
          step.data.id === messageId ||
          (step.data.messages && step.data.messages.includes(messageId))
        )) {
          relevantFlows.push({ flowId, step });
        }
      });
    });

    if (import.meta.env.DEV) {
      console.log(`Found in ${relevantFlows.length} flows:`, relevantFlows);
    }
    console.groupEnd();
  }

  reset() {
    this.flows.clear();
    this.currentFlowId = null;
  }

  export() {
    return {
      flows: Array.from(this.flows.entries()),
      timestamp: new Date().toISOString()
    };
  }
}

// Create global instance
const messageFlowDebugger = new MessageFlowDebugger();

// Export to window for debugging
if (typeof window !== 'undefined') {
  window.messageFlowDebugger = messageFlowDebugger;
  window.debugMessageFlow = (enabled = true) => {
    messageFlowDebugger.enabled = enabled;
    if (import.meta.env.DEV) {
      console.log(`Message flow debugging ${enabled ? 'enabled' : 'disabled'}`);
    }
  };
  window.findDuplicates = (messages) => messageFlowDebugger.findDuplicateMessages(messages);
  window.traceMessage = (messageId) => messageFlowDebugger.traceMessagePath(messageId);
}

export default messageFlowDebugger; 