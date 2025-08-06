// Core SDK exports
export { SniperForgeSDK } from './sdk/sniperforge-sdk';
export { TCPClient } from './sdk/tcp-client';

// Types exports
export * from './types';

// React hooks (if React is available)
export { useSniperForge } from './hooks/use-sniperforge';

// Vue composables (if Vue is available)
export { useSniperForgeComposable } from './composables/use-sniperforge';

// Utilities
export * from './utils';

// Version
export const VERSION = '1.0.0';
