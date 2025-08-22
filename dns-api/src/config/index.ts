import dotenv from 'dotenv';
import { AppConfig } from '../types';

dotenv.config();

const getEnvVar = (name: string, defaultValue?: string): string => {
  const value = process.env[name];
  if (!value && !defaultValue) {
    throw new Error(`Environment variable ${name} is required`);
  }
  return value || defaultValue!;
};

const getEnvNumber = (name: string, defaultValue: number): number => {
  const value = process.env[name];
  return value ? parseInt(value, 10) : defaultValue;
};

const getEnvArray = (name: string, defaultValue: string[] = []): string[] => {
  const value = process.env[name];
  return value ? value.split(',').map(item => item.trim()) : defaultValue;
};

export const config: AppConfig = {
  port: getEnvNumber('PORT', 3000),
  domain: getEnvVar('DOMAIN', 'example.com'),

  redis: {
    url: getEnvVar('REDIS_URL', 'redis://localhost:6379'),
    queueName: getEnvVar('REDIS_QUEUE_NAME', 'bip353_jobs'),
  },

  cloudflare: {
    apiToken: getEnvVar('CLOUDFLARE_API_TOKEN'),
    zoneId: getEnvVar('CLOUDFLARE_ZONE_ID'),
  },

  security: {
    apiKeys: getEnvArray('API_KEYS'),
    whitelistedIps: getEnvArray('WHITELISTED_IPS'),
  },

  worker: {
    maxRetries: getEnvNumber('WORKER_MAX_RETRIES', 3),
    retryDelayMs: getEnvNumber('WORKER_RETRY_DELAY_MS', 1000),
  },
};
