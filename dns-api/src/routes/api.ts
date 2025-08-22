import { Router, Request, Response } from 'express';
import Joi from 'joi';
import { QueueService } from '../services/queue';
import { authenticateApiKey, checkWhitelistedIp } from '../middleware/auth';
import { validateBody, validateParams } from '../middleware/validation';
import { createUsernameSchema, requestIdSchema } from '../validation/schemas';
import { logger } from '../utils/logger';
import { config } from '../config';
import {
  CreateUsernameRequest,
  CreateUsernameResponse,
  StatusResponse,
  ErrorResponse,
  JobStatus,
} from '../types';

const router = Router();
const queueService = new QueueService();

// Initialize queue service connection
queueService.connect().catch(error => {
  logger.error('Failed to connect queue service', { error: error.message });
  process.exit(1);
});

/**
 * @route POST /api/username
 * @desc Create a new BIP353 username
 * @access Private (API Key required)
 */
router.post(
  '/username',
  checkWhitelistedIp,
  authenticateApiKey,
  validateBody(createUsernameSchema),
  async (
    req: Request<
      {},
      CreateUsernameResponse | ErrorResponse,
      CreateUsernameRequest
    >,
    res: Response<CreateUsernameResponse | ErrorResponse>
  ) => {
    const { username, offer } = req.body;

    try {
      const requestId = await queueService.addJob(username, offer);

      logger.info('Username request queued', { username, requestId });
      const bip353Address = `${username}@${config.domain}`;
      const estimatedCompletionTime = new Date(
        Date.now() + 30000 // 30 seconds estimate
      ).toISOString();

      res.status(202).json({
        success: true,
        data: {
          requestId,
          status: JobStatus.PENDING,
          bip353Address,
          estimatedCompletionTime,
        },
      });
    } catch (error) {
      logger.error('Username request failed', {
        error: error instanceof Error ? error.message : String(error),
        username,
      });

      res.status(500).json({
        success: false,
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Failed to process username creation request',
        },
      });
    }
  }
);

/**
 * @route GET /api/status/:requestId
 * @desc Get the status of a username creation request
 * @access Private (API Key required)
 */
router.get(
  '/status/:requestId',
  checkWhitelistedIp,
  authenticateApiKey,
  validateParams(Joi.object({ requestId: requestIdSchema })),
  async (
    req: Request<{ requestId: string }>,
    res: Response<StatusResponse | ErrorResponse>
  ) => {
    const { requestId } = req.params;

    try {
      const job = await queueService.getJobStatus(requestId);

      if (!job) {
        res.status(404).json({
          success: false,
          error: {
            code: 'REQUEST_NOT_FOUND',
            message: 'No request found with the provided ID',
          },
        });
        return;
      }

      res.json({
        success: true,
        data: {
          requestId: job.id,
          status: job.status,
          bip353Address: job.metadata.bip353Address,
          createdAt: job.metadata.createdAt,
          ...(job.result?.completedAt && {
            completedAt: job.result.completedAt,
          }),
          ...(job.error?.message && { error: job.error.message }),
          ...(job.result?.recordId && { recordId: job.result.recordId }),
        },
      });
    } catch (error) {
      logger.error('Failed to get job status', {
        error: error instanceof Error ? error.message : String(error),
        requestId,
      });

      res.status(500).json({
        success: false,
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Failed to retrieve request status',
        },
      });
    }
  }
);

export { router as apiRoutes };
