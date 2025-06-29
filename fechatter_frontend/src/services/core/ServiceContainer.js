/**
 * 🔄 REFACTOR: Service Container & Dependency Injection
 * 
 * Implements clean architecture principles:
 * - Dependency Inversion: High-level modules don't depend on low-level modules
 * - Single Responsibility: Each service has one clear purpose
 * - Open/Closed: Easy to extend with new services
 * - Testable: All dependencies can be mocked
 */

export class ServiceContainer {
  constructor() {
    this.services = new Map();
    this.singletons = new Map();
    this.factories = new Map();
    this.aliases = new Map();

    // Configuration for service creation
    this.config = {
      enableLazyLoading: true,
      enableSingletonValidation: true,
      enableCircularDependencyDetection: true
    };

    // Track service creation for debugging
    this.creationStack = [];
    this.dependencyGraph = new Map();
  }

  /**
   * 🏭 REGISTER: Register service factories
   */
  register(name, factory, options = {}) {
    const serviceConfig = {
      factory,
      singleton: options.singleton ?? true,
      dependencies: options.dependencies || [],
      lazy: options.lazy ?? this.config.enableLazyLoading,
      tags: options.tags || []
    };

    this.factories.set(name, serviceConfig);

    // Register aliases
    if (options.aliases) {
      options.aliases.forEach(alias => {
        this.aliases.set(alias, name);
      });
    }

    // Build dependency graph for validation
    this.dependencyGraph.set(name, serviceConfig.dependencies);

    if (this.config.enableCircularDependencyDetection) {
      this.validateDependencies(name);
    }

    return this;
  }

  /**
   * RESOLVE: Get service instance with dependency injection
   */
  get(name) {
    // Resolve alias
    const serviceName = this.aliases.get(name) || name;

    // Check for circular dependency
    if (this.creationStack.includes(serviceName)) {
      throw new ServiceError(
        `Circular dependency detected: ${this.creationStack.join(' -> ')} -> ${serviceName}`
      );
    }

    // Return existing singleton
    if (this.singletons.has(serviceName)) {
      return this.singletons.get(serviceName);
    }

    // Get factory configuration
    const serviceConfig = this.factories.get(serviceName);
    if (!serviceConfig) {
      throw new ServiceError(`Service '${serviceName}' not found. Available services: ${Array.from(this.factories.keys()).join(', ')}`);
    }

    // Create service instance
    return this.createService(serviceName, serviceConfig);
  }

  /**
   * Create service instance with dependency injection
   */
  createService(name, config) {
    this.creationStack.push(name);

    try {
      // Resolve dependencies
      const dependencies = {};
      for (const depName of config.dependencies) {
        dependencies[depName] = this.get(depName);
      }

      // Create instance
      const instance = config.factory(dependencies, this);

      // Store singleton if configured
      if (config.singleton) {
        this.singletons.set(name, instance);
      }

      return instance;

    } finally {
      this.creationStack.pop();
    }
  }

  /**
   * 🧪 TESTING: Replace service for testing
   */
  mock(name, mockInstance) {
    this.singletons.set(name, mockInstance);
    return this;
  }

  /**
   * UTILITIES
   */
  has(name) {
    const serviceName = this.aliases.get(name) || name;
    return this.factories.has(serviceName) || this.singletons.has(serviceName);
  }

  clear() {
    // Call cleanup methods on services if they have them
    for (const [name, instance] of this.singletons.entries()) {
      if (typeof instance?.destroy === 'function') {
        try {
          instance.destroy();
        } catch (error) {
          console.warn(`Error destroying service '${name}':`, error);
        }
      }
    }

    this.services.clear();
    this.factories.clear();
    this.singletons.clear();
    this.aliases.clear();
    this.creationStack = [];
    this.dependencyGraph.clear();
  }

  getRegistered() {
    return Array.from(this.factories.keys());
  }

  getDependencyGraph() {
    const graph = {};
    for (const [name, { config }] of this.factories.entries()) {
      graph[name] = config.dependencies;
    }
    return graph;
  }

  /**
   * Validate dependency graph for circular dependencies
   */
  validateDependencies(startService) {
    const visited = new Set();
    const recursionStack = new Set();

    const hasCycle = (service) => {
      if (recursionStack.has(service)) {
        return true;
      }

      if (visited.has(service)) {
        return false;
      }

      visited.add(service);
      recursionStack.add(service);

      const dependencies = this.dependencyGraph.get(service) || [];
      for (const dep of dependencies) {
        if (hasCycle(dep)) {
          return true;
        }
      }

      recursionStack.delete(service);
      return false;
    };

    if (hasCycle(startService)) {
      throw new ServiceError(`Circular dependency detected starting from '${startService}'`);
    }
  }

  /**
   * Get container diagnostics
   */
  getDiagnostics() {
    return {
      registeredServices: Array.from(this.factories.keys()),
      singletonInstances: Array.from(this.singletons.keys()),
      aliases: Object.fromEntries(this.aliases),
      dependencyGraph: Object.fromEntries(this.dependencyGraph),
      config: this.config
    };
  }

  /**
   * Resolve service in parent container if not found locally
   */
  getFromParent(name) {
    return this.parent?.get(name);
  }

  /**
   * Override get method to support parent container lookup
   */
  get(name) {
    try {
      return super.get ? super.get(name) : this.getLocal(name);
    } catch (error) {
      if (this.parent) {
        return this.getFromParent(name);
      }
      throw error;
    }
  }

  /**
   * Get service only from local container
   */
  getLocal(name) {
    // Resolve alias
    const serviceName = this.aliases.get(name) || name;

    // Check for circular dependency
    if (this.creationStack.includes(serviceName)) {
      throw new ServiceError(
        `Circular dependency detected: ${this.creationStack.join(' -> ')} -> ${serviceName}`
      );
    }

    // Return existing singleton
    if (this.singletons.has(serviceName)) {
      return this.singletons.get(serviceName);
    }

    // Get factory configuration
    const serviceConfig = this.factories.get(serviceName);
    if (!serviceConfig) {
      throw new ServiceError(`Service '${serviceName}' not found. Available services: ${Array.from(this.factories.keys()).join(', ')}`);
    }

    // Create service instance
    return this.createService(serviceName, serviceConfig);
  }
}

/**
 * SERVICE FACTORY: Configure all application services
 */
export function createServiceContainer(stores = {}) {
  const container = new ServiceContainer();

  // LOGGING SERVICE: Cross-cutting concern
  container.register('logger', () => {
    const { createMessageTrackingLogger } = require('../logging/MessageTrackingLogger.js');
    return createMessageTrackingLogger();
  });

  // USER: USER DATA SERVICE: User resolution with DI
  container.register('userDataResolver', ({ logger }) => {
    const { createUserDataResolver } = require('../user/UserDataResolver.js');
    return createUserDataResolver(stores);
  }, {
    dependencies: ['logger']
  });

  // 🛡️ MESSAGE TRACKING SERVICE: Display guarantee
  container.register('messageTracker', ({ logger }) => {
    const { messageDisplayGuarantee } = require('../messageSystem/MessageDisplayGuarantee.js');

    // Inject logger into message display guarantee
    if (messageDisplayGuarantee.setLogger) {
      messageDisplayGuarantee.setLogger(logger);
    }

    return messageDisplayGuarantee;
  }, {
    dependencies: ['logger']
  });

  // SUBSCRIPTION: API SERVICE: HTTP communications
  container.register('apiService', () => {
    // Dynamic import to avoid circular dependencies
    return import('../api.js').then(module => module.default);
  });

  // 📥 MESSAGE FETCHER: Single responsibility data fetching
  container.register('messageFetcher', ({ apiService, logger }) => {
    return {
      async fetchMessages(chatId, options = {}) {
        const startTime = Date.now();
        logger.logFetchStart(chatId, options);

        try {
          const response = await apiService.get(`/chat/${chatId}/messages`, {
            params: { limit: options.limit || 15 },
            signal: options.abortSignal
          });

          const messages = response.data?.data || response.data || [];
          const duration = Date.now() - startTime;

          logger.logFetchSuccess(chatId, messages.length, duration);
          return messages;
        } catch (error) {
          logger.logError('message-fetching', error, { chatId, options });
          throw error;
        }
      }
    };
  }, {
    dependencies: ['apiService', 'logger']
  });

  // MESSAGE NORMALIZER: Data transformation
  container.register('messageNormalizer', ({ userDataResolver }) => {
    return {
      normalizeMessage(rawMsg, chatId) {
        return {
          id: rawMsg.id,
          content: rawMsg.content || '',
          sender_id: rawMsg.sender_id,
          sender_name: userDataResolver.resolveUserName(rawMsg),
          sender: userDataResolver.createSenderObject(rawMsg),
          created_at: rawMsg.created_at,
          chat_id: parseInt(chatId),
          status: 'sent',
          files: this.standardizeFiles(rawMsg.files || []),
          mentions: rawMsg.mentions || [],
          reply_to: rawMsg.reply_to || null
        };
      },

      normalizeMessages(rawMessages, chatId) {
        const normalized = rawMessages.map(msg => this.normalizeMessage(msg, chatId));

        // Sort chronologically (oldest first)
        return normalized.sort((a, b) => {
          const timeA = new Date(a.created_at).getTime();
          const timeB = new Date(b.created_at).getTime();
          return timeA - timeB;
        });
      },

      standardizeFiles(files) {
        if (!files || !Array.isArray(files)) return [];

        return files.map(file => {
          if (typeof file === 'string') {
            return {
              id: Date.now() + Math.random(),
              url: file,
              filename: file.split('/').pop() || 'Unknown File',
              mime_type: this.getMimeTypeFromUrl(file),
              size: 0
            };
          }

          return {
            id: file.id || Date.now() + Math.random(),
            url: file.url || file.path || '',
            filename: file.filename || file.name || 'Unknown File',
            mime_type: file.mime_type || file.type || 'application/octet-stream',
            size: file.size || 0
          };
        });
      },

      getMimeTypeFromUrl(url) {
        const extension = url.split('.').pop()?.toLowerCase();
        const mimeTypes = {
          'jpg': 'image/jpeg', 'jpeg': 'image/jpeg', 'png': 'image/png',
          'gif': 'image/gif', 'pdf': 'application/pdf', 'txt': 'text/plain',
          'mp4': 'video/mp4', 'mp3': 'audio/mpeg'
        };
        return mimeTypes[extension] || 'application/octet-stream';
      }
    };
  }, {
    dependencies: ['userDataResolver']
  });

  // 🔄 REFACTORED MESSAGE SERVICE: Orchestrates other services
  container.register('messageService', ({
    messageFetcher,
    messageNormalizer,
    messageTracker,
    userDataResolver,
    logger
  }) => {
    const { RefactoredMessageService } = require('./RefactoredMessageService.js');
    return new RefactoredMessageService({
      messageFetcher,
      messageNormalizer,
      messageTracker,
      userDataResolver,
      logger
    });
  }, {
    dependencies: ['messageFetcher', 'messageNormalizer', 'messageTracker', 'userDataResolver', 'logger']
  });

  return container;
}

/**
 * 🌍 GLOBAL CONTAINER: Singleton instance for the application
 */
let globalContainer = null;

export function getServiceContainer() {
  if (!globalContainer) {
    throw new Error('Service container not initialized. Call initializeServices() first.');
  }
  return globalContainer;
}

export function initializeServices(stores) {
  if (globalContainer) {
    console.warn('Services already initialized');
    return globalContainer;
  }

  globalContainer = createServiceContainer(stores);
  console.log('Service container initialized with services:', globalContainer.getRegistered());

  return globalContainer;
}

export function clearServices() {
  if (globalContainer) {
    globalContainer.clear();
    globalContainer = null;
  }
}

/**
 * 🧪 TESTING UTILITIES
 */
export function createTestContainer(mocks = {}) {
  const container = new ServiceContainer();

  // Register mocks
  for (const [name, mock] of Object.entries(mocks)) {
    container.register(name, () => mock);
  }

  return container;
}

/**
 * Service Error class for container-specific errors
 */
export class ServiceError extends Error {
  constructor(message) {
    super(message);
    this.name = 'ServiceError';
  }
}

/**
 * Testing utilities for service container
 */
export class ServiceContainerTestUtils {
  /**
   * Create a test container with mock services
   */
  static createTestContainer() {
    const container = new ServiceContainer();
    container.config.enableCircularDependencyDetection = false; // For easier testing
    return container;
  }

  /**
   * Create mock factory for testing
   */
  static mockFactory(mockInstance) {
    return () => mockInstance;
  }

  /**
   * Create mock service with common methods
   */
  static createMockService(methods = {}) {
    return {
      // Common mock methods
      initialize: jest?.fn() || (() => { }),
      destroy: jest?.fn() || (() => { }),
      // Custom methods
      ...methods
    };
  }

  /**
   * Spy on service creation for testing
   */
  static spyOnServiceCreation(container, serviceName) {
    const originalGet = container.get.bind(container);
    const spy = jest?.fn() || (() => { });

    container.get = function (name) {
      if (name === serviceName) {
        spy(name);
      }
      return originalGet(name);
    };

    return spy;
  }
}

/**
 * Global service container instance
 */
export const serviceContainer = new ServiceContainer();

/**
 * Decorator for automatic service injection
 */
export function inject(serviceName) {
  return function (target, propertyKey) {
    Object.defineProperty(target, propertyKey, {
      get() {
        return serviceContainer.get(serviceName);
      },
      configurable: true
    });
  };
}

/**
 * Helper function to register common service types
 */
export function registerService(name, factory, options = {}) {
  return serviceContainer.register(name, factory, options);
}

/**
 * Helper function to get service instances
 */
export function getService(name) {
  return serviceContainer.get(name);
} 