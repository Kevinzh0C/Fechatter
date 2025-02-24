/**
 * Simple Dependency Injection Container for Fechatter
 * 
 * Provides basic service container functionality for managing dependencies
 * across the application in a testable and maintainable way.
 */

class DIContainer {
  constructor() {
    this.services = new Map();
    this.singletons = new Map();
    this.factories = new Map();
  }

  /**
   * Register a service with the container
   * @param {string} name - Service name
   * @param {Function|Object} implementation - Service implementation
   * @param {Object} options - Registration options
   */
  register(name, implementation, options = {}) {
    const { singleton = false, factory = false } = options;

    if (factory) {
      this.factories.set(name, implementation);
    } else if (singleton) {
      this.services.set(name, { implementation, singleton: true });
    } else {
      this.services.set(name, { implementation, singleton: false });
    }

    return this;
  }

  /**
   * Register a singleton service
   * @param {string} name - Service name  
   * @param {Function|Object} implementation - Service implementation
   */
  singleton(name, implementation) {
    return this.register(name, implementation, { singleton: true });
  }

  /**
   * Register a factory function
   * @param {string} name - Factory name
   * @param {Function} factory - Factory function
   */
  factory(name, factory) {
    return this.register(name, factory, { factory: true });
  }

  /**
   * Resolve a service from the container
   * @param {string} name - Service name
   * @returns {Object} Service instance
   */
  resolve(name) {
    // Check singletons first
    if (this.singletons.has(name)) {
      return this.singletons.get(name);
    }

    // Check factories
    if (this.factories.has(name)) {
      const factory = this.factories.get(name);
      return factory(this);
    }

    // Check registered services
    if (this.services.has(name)) {
      const service = this.services.get(name);

      if (service.singleton) {
        // Create singleton instance
        const instance = typeof service.implementation === 'function'
          ? new service.implementation(this)
          : service.implementation;

        this.singletons.set(name, instance);
        return instance;
      } else {
        // Create new instance
        return typeof service.implementation === 'function'
          ? new service.implementation(this)
          : service.implementation;
      }
    }

    throw new Error(`Service '${name}' not found in container`);
  }

  /**
   * Check if a service is registered
   * @param {string} name - Service name
   * @returns {boolean}
   */
  has(name) {
    return this.services.has(name) ||
      this.factories.has(name) ||
      this.singletons.has(name);
  }

  /**
   * Install container services into Vue app
   * @param {Object} app - Vue app instance
   */
  install(app) {
    // Make container available via provide/inject
    app.provide('container', this);

    // Add global property for easier access
    app.config.globalProperties.$container = this;

    if (import.meta.env.DEV) {
      console.log('✅ DIContainer installed into Vue app');

      // Add debug helpers
      window.container = this;
      window.debugContainer = () => {
        console.group('🔧 DIContainer Debug Info');
        console.log('Registered Services:', Array.from(this.services.keys()));
        console.log('Singletons:', Array.from(this.singletons.keys()));
        console.log('Factories:', Array.from(this.factories.keys()));
        console.groupEnd();
      };
    }
  }

  /**
   * Clear all services (useful for testing)
   */
  clear() {
    this.services.clear();
    this.singletons.clear();
    this.factories.clear();
  }

  /**
   * Get container statistics
   * @returns {Object} Container stats
   */
  getStats() {
    return {
      services: this.services.size,
      singletons: this.singletons.size,
      factories: this.factories.size,
      total: this.services.size + this.factories.size
    };
  }
}

// Create container instance
const container = new DIContainer();

/**
 * Configure core services
 * @param {DIContainer} container - Container instance
 */
export function configureServices(container) {
  // Register core services here when needed

  // Example service registrations:
  // container.singleton('logger', Logger);
  // container.singleton('api', ApiService);
  // container.factory('eventBus', () => new EventBus());

  if (import.meta.env.DEV) {
    console.log('🔧 Core services configured');
  }
}

// Export container and configuration
export { container, DIContainer };
export default container; 