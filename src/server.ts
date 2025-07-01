import express from 'express';
import helmet from 'helmet';
import cors from 'cors';
import { config } from './config';
import { logger } from './utils/logger';
import { apiRoutes } from './routes/api';
import { ErrorResponse } from './types';

const app = express();

// Security middleware
app.use(helmet());
app.use(cors());
app.use(express.json({ limit: '1mb' }));

// JSON parsing error handler (must be after express.json)
app.use(
  (
    err: any,
    req: express.Request,
    res: express.Response<ErrorResponse>,
    next: express.NextFunction
  ): void => {
    if (err instanceof SyntaxError && 'body' in err) {
      res.status(400).json({
        success: false,
        error: {
          code: 'INVALID_JSON',
          message: 'Invalid JSON in request body',
        },
      });
      return;
    }
    next(err);
  }
);

// Trust proxy for accurate IP addresses
app.set('trust proxy', 1);

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: process.env['npm_package_version'] || '1.0.0',
  });
});

// API routes
app.use('/api', apiRoutes);

// 404 handler
app.use('*', (req, res: express.Response<ErrorResponse>) => {
  return res.status(404).json({
    success: false,
    error: {
      code: 'NOT_FOUND',
      message: 'The requested endpoint does not exist',
    },
  });
});

// Global error handler
app.use(
  (
    err: Error,
    req: express.Request,
    res: express.Response<ErrorResponse>,
    next: express.NextFunction
  ) => {
    logger.error('Unhandled error', {
      error: err.message,
      stack: err.stack,
      path: req.path,
      method: req.method,
    });

    res.status(500).json({
      success: false,
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An unexpected error occurred',
      },
    });
  }
);

// Graceful shutdown handlers
const gracefulShutdown = (signal: string) => {
  logger.info(`Received ${signal}, shutting down gracefully`);
  process.exit(0);
};

process.on('SIGTERM', () => gracefulShutdown('SIGTERM'));
process.on('SIGINT', () => gracefulShutdown('SIGINT'));

// Uncaught exception handler
process.on('uncaughtException', error => {
  logger.error('Uncaught exception', {
    error: error.message,
    stack: error.stack,
  });
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  logger.error('Unhandled promise rejection', { reason, promise });
  process.exit(1);
});

// Start server
const startServer = async (): Promise<void> => {
  try {
    app.listen(config.port, () => {
      logger.info(`BIP353 API Server started on port ${config.port}`);
    });
  } catch (error) {
    logger.error('Server startup failed', {
      error: error instanceof Error ? error.message : String(error),
    });
    process.exit(1);
  }
};

startServer();
