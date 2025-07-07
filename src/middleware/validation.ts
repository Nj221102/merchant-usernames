import { Request, Response, NextFunction } from 'express';
import Joi from 'joi';
import { ErrorResponse } from '../types';

export const validateBody = <T>(schema: Joi.ObjectSchema<T>) => {
  return (
    req: Request,
    res: Response<ErrorResponse>,
    next: NextFunction
  ): void => {
    const { error } = schema.validate(req.body);

    if (error) {
      res.status(400).json({
        success: false,
        error: {
          code: 'VALIDATION_ERROR',
          message: error.details[0]?.message || 'Validation failed',
          details: {
            field: error.details[0]?.path?.join('.'),
            value: error.details[0]?.context?.value,
          },
        },
      });
      return;
    }

    next();
  };
};

export const validateParams = <T>(schema: Joi.ObjectSchema<T>) => {
  return (
    req: Request,
    res: Response<ErrorResponse>,
    next: NextFunction
  ): void => {
    const { error } = schema.validate(req.params);

    if (error) {
      res.status(400).json({
        success: false,
        error: {
          code: 'VALIDATION_ERROR',
          message: error.details[0]?.message || 'Parameter validation failed',
          details: {
            field: error.details[0]?.path?.join('.'),
            value: error.details[0]?.context?.value,
          },
        },
      });
      return;
    }

    next();
  };
};
