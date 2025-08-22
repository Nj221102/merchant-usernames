import Joi from 'joi';
import { CreateUsernameRequest } from '../types';

export const createUsernameSchema = Joi.object<CreateUsernameRequest>({
  username: Joi.string().alphanum().min(3).max(30).required().messages({
    'string.alphanum': 'Username must contain only alphanumeric characters',
    'string.min': 'Username must be at least 3 characters long',
    'string.max': 'Username must not exceed 30 characters',
    'any.required': 'Username is required',
  }),

  offer: Joi.string()
    .pattern(/^lno1[a-z0-9]+$/)
    .required()
    .messages({
      'string.pattern.base':
        'Offer must be a valid BOLT12 offer starting with "lno1"',
      'any.required': 'BOLT12 offer is required',
    }),
});

export const requestIdSchema = Joi.string().uuid().required().messages({
  'string.guid': 'Request ID must be a valid UUID',
  'any.required': 'Request ID is required',
});
