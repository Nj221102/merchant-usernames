import { createClient, RedisClientType } from 'redis';
import { v4 as uuidv4 } from 'uuid';
import { config } from '../config';
import { logger } from '../utils/logger';
import { Job, JobStatus } from '../types';

export class QueueService {
  private client: RedisClientType | null = null;
  private readonly queueName = config.redis.queueName;
  private readonly statusKeyPrefix = 'job_status:';

  async connect(): Promise<void> {
    try {
      this.client = createClient({
        url: config.redis.url,
      });

      this.client.on('error', err => {
        logger.error('Redis connection error', { error: err.message });
      });

      await this.client.connect();
      logger.info('Connected to Redis successfully');
    } catch (error) {
      logger.error('Failed to connect to Redis', {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  async addJob(username: string, offer: string): Promise<string> {
    if (!this.client) {
      throw new Error('Redis client not connected');
    }

    const jobId = uuidv4();
    const now = new Date().toISOString();
    const bip353Address = `${username}@${config.domain}`;

    const job: Job = {
      id: jobId,
      status: JobStatus.PENDING,
      data: { username, offer },
      metadata: {
        createdAt: now,
        updatedAt: now,
        retryCount: 0,
        bip353Address,
      },
    };

    try {
      // Add job to queue
      await this.client.lPush(this.queueName, JSON.stringify(job));

      // Store job status for lookup
      await this.client.setEx(
        `${this.statusKeyPrefix}${jobId}`,
        3600, // 1 hour TTL
        JSON.stringify(job)
      );

      logger.info('Job added to queue', { jobId, username });
      return jobId;
    } catch (error) {
      logger.error('Failed to add job to queue', {
        error: error instanceof Error ? error.message : String(error),
        jobId,
        username,
      });
      throw error;
    }
  }

  async getJob(): Promise<Job | null> {
    if (!this.client) {
      throw new Error('Redis client not connected');
    }

    try {
      const result = await this.client.brPop(this.queueName, 0);

      if (result) {
        const job: Job = JSON.parse(result.element.toString());

        // Update status to processing
        await this.updateJobStatus(job.id, JobStatus.PROCESSING);

        return job;
      }

      return null;
    } catch (error) {
      logger.error('Failed to get job from queue', {
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  async updateJobStatus(
    jobId: string,
    status: JobStatus,
    result?: { recordId: string; completedAt: string },
    error?: { message: string; code: string; occurredAt: string }
  ): Promise<void> {
    if (!this.client) {
      throw new Error('Redis client not connected');
    }

    try {
      const jobKey = `${this.statusKeyPrefix}${jobId}`;
      const jobData = await this.client.get(jobKey);

      if (!jobData) {
        logger.warn('Job not found for status update', { jobId });
        return;
      }

      const job: Job = JSON.parse(jobData);
      job.status = status;
      job.metadata.updatedAt = new Date().toISOString();

      if (result) {
        job.result = result;
      }

      if (error) {
        job.error = error;
      }

      await this.client.setEx(jobKey, 3600, JSON.stringify(job));
      if (status === JobStatus.COMPLETED || status === JobStatus.FAILED) {
        logger.info('Job status updated', { jobId, status });
      }
    } catch (err) {
      logger.error('Failed to update job status', {
        error: err instanceof Error ? err.message : String(err),
        jobId,
        status,
      });
      throw err;
    }
  }

  async getJobStatus(jobId: string): Promise<Job | null> {
    if (!this.client) {
      throw new Error('Redis client not connected');
    }

    try {
      const jobData = await this.client.get(`${this.statusKeyPrefix}${jobId}`);

      if (!jobData) {
        return null;
      }

      return JSON.parse(jobData);
    } catch (error) {
      logger.error('Failed to get job status', {
        error: error instanceof Error ? error.message : String(error),
        jobId,
      });
      throw error;
    }
  }

  async requeueJob(job: Job): Promise<void> {
    if (!this.client) {
      throw new Error('Redis client not connected');
    }

    job.metadata.retryCount += 1;
    job.metadata.updatedAt = new Date().toISOString();
    job.status = JobStatus.PENDING;

    try {
      await this.client.lPush(this.queueName, JSON.stringify(job));
      await this.updateJobStatus(job.id, JobStatus.PENDING);

      logger.info('Job requeued for retry', {
        jobId: job.id,
        attempt: job.metadata.retryCount,
      });
    } catch (error) {
      logger.error('Failed to requeue job', {
        error: error instanceof Error ? error.message : String(error),
        jobId: job.id,
      });
      throw error;
    }
  }

  async disconnect(): Promise<void> {
    if (this.client) {
      await this.client.disconnect();
      this.client = null;
      logger.info('Disconnected from Redis');
    }
  }
}
