import { QueueService } from './services/queue';
import { CloudflareService } from './services/cloudflare';
import { logger } from './utils/logger';
import { config } from './config';
import { Job, JobStatus } from './types';

class Worker {
  private queueService = new QueueService();
  private cloudflareService = new CloudflareService();
  private isRunning = false;

  async start(): Promise<void> {
    try {
      await this.queueService.connect();
      this.isRunning = true;

      logger.info('Worker started successfully');

      while (this.isRunning) {
        try {
          await this.processJobs();
        } catch (error) {
          logger.error('Error in job processing loop', {
            error: error instanceof Error ? error.message : String(error),
          });

          // Wait before retrying to avoid rapid error loops
          await this.sleep(5000);
        }
      }
    } catch (error) {
      logger.error('Failed to start worker', {
        error: error instanceof Error ? error.message : String(error),
      });
      process.exit(1);
    }
  }

  private async processJobs(): Promise<void> {
    const job = await this.queueService.getJob();

    if (!job) {
      return;
    }

    try {
      await this.processJob(job);
    } catch (error) {
      await this.handleJobFailure(job, error);
    }
  }

  private async processJob(job: Job): Promise<void> {
    const { username, offer } = job.data;

    try {
      // Create the DNS TXT record
      const result = await this.cloudflareService.createTxtRecord(
        username,
        offer
      );

      if (result.success) {
        const completedAt = new Date().toISOString();

        await this.queueService.updateJobStatus(job.id, JobStatus.COMPLETED, {
          recordId: result.recordId,
          completedAt,
        });

        logger.info('DNS record created', {
          jobId: job.id,
          username,
          recordId: result.recordId,
          bip353Address: job.metadata.bip353Address,
        });
      } else {
        throw new Error(result.error);
      }
    } catch (error) {
      // Re-throw errors for retry logic
      throw error;
    }
  }

  private async handleJobFailure(job: Job, error: unknown): Promise<void> {
    const errorMessage = error instanceof Error ? error.message : String(error);

    logger.error('Job processing failed', {
      jobId: job.id,
      username: job.data.username,
      error: errorMessage,
      retryCount: job.metadata.retryCount,
    });

    // Should we retry this job?
    if (job.metadata.retryCount < config.worker.maxRetries) {
      const nextRetryCount = job.metadata.retryCount + 1;

      logger.info(
        `Retrying job (${nextRetryCount}/${config.worker.maxRetries})`,
        {
          jobId: job.id,
        }
      );

      // Exponential backoff
      const delay =
        config.worker.retryDelayMs * Math.pow(2, job.metadata.retryCount);
      await this.sleep(delay);

      // Requeue the job
      await this.queueService.requeueJob(job);
    } else {
      // Mark job as permanently failed
      const failedAt = new Date().toISOString();

      await this.queueService.updateJobStatus(
        job.id,
        JobStatus.FAILED,
        undefined,
        {
          message: errorMessage,
          code: 'MAX_RETRIES_EXCEEDED',
          occurredAt: failedAt,
        }
      );

      logger.error('Job failed permanently after maximum retries', {
        jobId: job.id,
        username: job.data.username,
        maxRetries: config.worker.maxRetries,
        finalError: errorMessage,
      });
    }
  }

  private async sleep(milliseconds: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, milliseconds));
  }

  async stop(): Promise<void> {
    logger.info('Stopping worker...');
    this.isRunning = false;
    await this.queueService.disconnect();
    logger.info('Worker stopped');
  }
}

// Create and start worker
const worker = new Worker();

// Graceful shutdown handlers
const gracefulShutdown = async (signal: string): Promise<void> => {
  logger.info(`Received ${signal}, shutting down worker gracefully`);
  await worker.stop();
  process.exit(0);
};

process.on('SIGTERM', () => gracefulShutdown('SIGTERM'));
process.on('SIGINT', () => gracefulShutdown('SIGINT'));

// Error handlers
process.on('uncaughtException', error => {
  logger.error('Uncaught exception in worker', {
    error: error.message,
    stack: error.stack,
  });
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  logger.error('Unhandled promise rejection in worker', { reason, promise });
  process.exit(1);
});

// Start the worker
worker.start();
