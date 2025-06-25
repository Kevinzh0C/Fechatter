/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_URL: string
  readonly VITE_DEBUG: string
  readonly VITE_APP_TITLE: string
  readonly BASE_URL: string
  readonly DEV: boolean
  readonly PROD: boolean
  readonly SSR: boolean
  readonly MODE: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare global {
  interface Window {
    gtag?: (command: string, eventName: string, parameters?: Record<string, any>) => void
  }
}

export { } 