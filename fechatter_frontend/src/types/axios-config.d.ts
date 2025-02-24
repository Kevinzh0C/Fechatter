import 'axios';

declare module 'axios' {
  export interface AxiosRequestConfig {
    /**
     * When true, skip the 401 token refresh interceptor and propagate 401 error directly.
     */
    skipAuthRefresh?: boolean;

    /**
     * Internal flag for retry after refresh.
     */
    _retry?: boolean;
  }
} 