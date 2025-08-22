import { Request, Response, NextFunction } from 'express';
import { config } from '../config';
import { logger } from '../utils/logger';
import { ErrorResponse } from '../types';

export const authenticateApiKey = (
  req: Request,
  res: Response<ErrorResponse>,
  next: NextFunction
): void => {
  const authHeader = req.headers.authorization;

  if (!authHeader?.startsWith('Bearer ')) {
    logger.warn(
      'Authentication failed: Missing or invalid Authorization header',
      {
        ip: req.ip,
        path: req.path,
      }
    );

    res.status(401).json({
      success: false,
      error: {
        code: 'UNAUTHORIZED',
        message: 'Please provide a valid API key in the Authorization header',
      },
    });
    return;
  }

  const apiKey = authHeader.substring(7);

  if (!config.security.apiKeys.includes(apiKey)) {
    logger.warn('Authentication failed: Invalid API key', {
      ip: req.ip,
      path: req.path,
    });

    res.status(401).json({
      success: false,
      error: {
        code: 'INVALID_API_KEY',
        message: 'The provided API key is not valid',
      },
    });
    return;
  }

  logger.debug('Authentication successful', { ip: req.ip });
  next();
};

export const checkWhitelistedIp = (
  req: Request,
  res: Response<ErrorResponse>,
  next: NextFunction
): void => {
  const { whitelistedIps } = config.security;

  // Skip IP check if no whitelist is configured
  if (whitelistedIps.length === 0) {
    next();
    return;
  }

  const clientIp = req.ip || req.socket.remoteAddress || 'unknown';

  if (!whitelistedIps.includes(clientIp)) {
    logger.warn('IP access denied', {
      ip: clientIp,
      path: req.path,
    });

    res.status(403).json({
      success: false,
      error: {
        code: 'IP_NOT_WHITELISTED',
        message: 'Access denied from this IP address',
      },
    });
    return;
  }

  next();
};
