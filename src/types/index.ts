export interface CreateUsernameRequest {
  username: string;
  offer: string;
}

export interface CreateUsernameResponse {
  success: true;
  data: {
    requestId: string;
    status: JobStatus;
    bip353Address: string;
    estimatedCompletionTime: string;
  };
}

export interface StatusResponse {
  success: true;
  data: {
    requestId: string;
    status: JobStatus;
    bip353Address: string;
    createdAt: string;
    completedAt?: string;
    error?: string;
    recordId?: string;
  };
}

export interface ErrorResponse {
  success: false;
  error: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
}

export enum JobStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  COMPLETED = 'completed',
  FAILED = 'failed',
}

export interface Job {
  id: string;
  status: JobStatus;
  data: {
    username: string;
    offer: string;
  };
  metadata: {
    createdAt: string;
    updatedAt: string;
    retryCount: number;
    bip353Address: string;
  };
  result?: {
    recordId: string;
    completedAt: string;
  };
  error?: {
    message: string;
    code: string;
    occurredAt: string;
  };
}

export interface CloudflareRecord {
  id: string;
  name: string;
  type: string;
  content: string;
  ttl: number;
}

export interface CloudflareResponse<T = unknown> {
  success: boolean;
  errors: Array<{
    code: number;
    message: string;
  }>;
  messages: string[];
  result: T;
}

export interface AppConfig {
  port: number;
  domain: string;
  redis: {
    url: string;
    queueName: string;
  };
  cloudflare: {
    apiToken: string;
    zoneId: string;
  };
  security: {
    apiKeys: string[];
    whitelistedIps: string[];
  };
  worker: {
    maxRetries: number;
    retryDelayMs: number;
  };
}
