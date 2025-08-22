import axios, { AxiosResponse } from 'axios';
import { config } from '../config';
import { logger } from '../utils/logger';
import { CloudflareResponse, CloudflareRecord } from '../types';

export class CloudflareService {
  private readonly baseUrl = 'https://api.cloudflare.com/client/v4';
  private readonly apiToken = config.cloudflare.apiToken;
  private readonly zoneId = config.cloudflare.zoneId;

  async createTxtRecord(
    username: string,
    offer: string
  ): Promise<
    { success: true; recordId: string } | { success: false; error: string }
  > {
    const recordName = `${username}.user._bitcoin-payment`;
    const recordContent = `"bitcoin:?lno=${offer}"`;

    try {
      const response: AxiosResponse<CloudflareResponse<CloudflareRecord>> =
        await axios.post(
          `${this.baseUrl}/zones/${this.zoneId}/dns_records`,
          {
            type: 'TXT',
            name: recordName,
            content: recordContent,
            ttl: 300,
          },
          {
            headers: {
              Authorization: `Bearer ${this.apiToken}`,
              'Content-Type': 'application/json',
            },
          }
        );

      if (response.data.success) {
        return {
          success: true,
          recordId: response.data.result.id,
        };
      } else {
        const errorMessage = response.data.errors
          .map(err => `${err.message} (${err.code})`)
          .join(', ');

        logger.error('Cloudflare API returned errors', {
          username,
          errors: response.data.errors,
        });

        return {
          success: false,
          error: errorMessage,
        };
      }
    } catch (error) {
      if (axios.isAxiosError(error)) {
        logger.error('Cloudflare API request failed', {
          username,
          status: error.response?.status,
          data: error.response?.data,
        });

        return {
          success: false,
          error: error.response?.data?.errors?.[0]?.message || error.message,
        };
      }

      logger.error('Network error when calling Cloudflare API', {
        username,
        error: error instanceof Error ? error.message : String(error),
      });

      return {
        success: false,
        error: 'Network error occurred while creating DNS record',
      };
    }
  }
}
